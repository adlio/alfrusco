{-# LANGUAGE FunctionalDependencies #-}
{-# LANGUAGE OverloadedStrings #-}

-- | Runnable typeclass and execute functions for Alfred workflows.
module Alfrusco.Runnable
  ( Runnable (..)
  , execute
  , executeAsync
  ) where

import Data.IORef (readIORef)
import System.Exit (exitFailure)
import System.IO (Handle, hPutStrLn, stderr)

import Alfrusco.Config (ConfigProvider (..))
import Alfrusco.Error (IsWorkflowError (..))
import Alfrusco.Response (Response (..), writeResponse)
import Alfrusco.SortAndFilter (filterAndSortItems)
import Alfrusco.Workflow

-- | Typeclass for runnable workflow actions.
class IsWorkflowError e => Runnable r e | r -> e where
  run :: r -> Workflow -> IO (Either e ())

-- | Execute a runnable action using the given config provider and output handle.
--
-- Logic:
-- 1. Get config from the provider (exit on error).
-- 2. Create a workflow (exit on error).
-- 3. Call run on the runnable.
-- 4. On Left, prepend the error item to the response.
-- 5. Finalize: apply sort/filter if keyword set, write response JSON to handle.
execute :: (ConfigProvider p, Runnable r e) => p -> r -> Handle -> IO ()
execute provider runnable writer = do
  wf <- setupWorkflow provider
  result <- run runnable wf
  case result of
    Left e -> prependItem wf (errorItem e)
    Right () -> pure ()
  finalizeWorkflow wf writer

-- | Async variant of execute. In Haskell, IO is already concurrent-capable,
-- so this has the same implementation as execute.
executeAsync :: (ConfigProvider p, Runnable r e) => p -> r -> Handle -> IO ()
executeAsync = execute

-- | Internal: set up a workflow from a config provider.
-- Exits the process on failure.
setupWorkflow :: ConfigProvider p => p -> IO Workflow
setupWorkflow provider = do
  eConfig <- getConfig provider
  case eConfig of
    Left e -> do
      hPutStrLn stderr ("Error loading config: " ++ show e)
      exitFailure
    Right config -> do
      wf <- newWorkflow config
      pure wf

-- | Internal: finalize a workflow by applying filtering and writing the response.
finalizeWorkflow :: Workflow -> Handle -> IO ()
finalizeWorkflow wf writer = do
  shouldFilter <- readIORef (wfSortAndFilter wf)
  response <- readIORef (wfResponse wf)
  finalResponse <- if shouldFilter
    then do
      mKeyword <- readIORef (wfKeyword wf)
      case mKeyword of
        Just kw -> pure response {responseItems = filterAndSortItems (responseItems response) kw}
        Nothing -> pure response
    else pure response
  writeResponse writer finalResponse
