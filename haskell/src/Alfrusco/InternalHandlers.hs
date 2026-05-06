{-# LANGUAGE OverloadedStrings #-}

-- | Internal command handlers for special Alfred workflow operations.
-- Handles clipboard commands and workflow directory navigation.
module Alfrusco.InternalHandlers
  ( handle
  , handleWorkflowDirOpen
  , parseWorkflowCommand
  , WorkflowCommand (..)
  , createWorkflowCommandSuggestions
  , openPath
  ) where

import Control.Exception (IOException, try)
import Data.Text (Text)
import Data.Text qualified as Text
import System.Environment (getArgs, lookupEnv)
import System.Process (callProcess)

import Alfrusco.Clipboard (handleClipboard)
import Alfrusco.Item (Item, item, withAutocomplete, withSticky, withSubtitle, withValid)
import Alfrusco.Workflow (Workflow, appendItems, cacheDir)

-- | Parsed workflow command type.
data WorkflowCommand
  = OpenCache
  | OpenData
  | OpenLog
  | ShowSuggestions
  | NoCommand
  deriving (Show, Eq)

-- | Handle special commands based on environment variables and command-line arguments.
-- Returns True if a command was handled and the process should exit.
handle :: Workflow -> IO Bool
handle wf = do
  clipboardHandled <- handleClipboard
  if clipboardHandled
    then pure True
    else handleWorkflowDirOpen wf

-- | Handle workflow directory open commands based on query in command-line args.
-- Returns True if a directory was opened, False otherwise.
handleWorkflowDirOpen :: Workflow -> IO Bool
handleWorkflowDirOpen wf = do
  mQuery <- extractQueryFromArgs
  case mQuery of
    Nothing -> pure False
    Just query -> do
      let command = parseWorkflowCommand query
      executeWorkflowCommand command wf

-- | Extract the query from command-line arguments (last argument).
extractQueryFromArgs :: IO (Maybe Text)
extractQueryFromArgs = do
  args <- getArgs
  case args of
    [] -> pure Nothing
    _  -> pure (Just (Text.pack (last args)))

-- | Parse a query string to determine if it is a workflow command.
parseWorkflowCommand :: Text -> WorkflowCommand
parseWorkflowCommand query =
  let trimmed = Text.strip query
  in case trimmed of
    "workflow:cache"   -> OpenCache
    "workflow:data"    -> OpenData
    "workflow:openlog" -> OpenLog
    _ | Text.isPrefixOf "work" trimmed -> ShowSuggestions
      | otherwise -> NoCommand

-- | Execute a workflow command, returning True if the process should exit.
executeWorkflowCommand :: WorkflowCommand -> Workflow -> IO Bool
executeWorkflowCommand OpenCache _wf = openDirectoryFromEnv "alfred_workflow_cache"
executeWorkflowCommand OpenData _wf = openDirectoryFromEnv "alfred_workflow_data"
executeWorkflowCommand OpenLog wf = openLogFile wf
executeWorkflowCommand ShowSuggestions wf = do
  let suggestions = createWorkflowCommandSuggestions
  appendItems wf suggestions
  pure False
executeWorkflowCommand NoCommand _wf = pure False

-- | Create workflow command suggestion items.
createWorkflowCommandSuggestions :: [Item]
createWorkflowCommandSuggestions =
  [ withSticky True
    $ withValid False
    $ withAutocomplete "workflow:data"
    $ withSubtitle "workflow:data"
    $ item "Open the workflow data directory"
  , withSticky True
    $ withValid False
    $ withAutocomplete "workflow:cache"
    $ withSubtitle "workflow:cache"
    $ item "Open the workflow cache directory"
  , withSticky True
    $ withValid False
    $ withAutocomplete "workflow:openlog"
    $ withSubtitle "workflow:openlog"
    $ item "Open the workflow log file"
  ]

-- | Open a directory from an environment variable.
openDirectoryFromEnv :: String -> IO Bool
openDirectoryFromEnv envVar = do
  mPath <- lookupEnv envVar
  case mPath of
    Just path -> openPath path
    Nothing   -> pure False

-- | Open the log file.
openLogFile :: Workflow -> IO Bool
openLogFile wf = do
  mLogPath <- lookupEnv "alfred_workflow_log"
  case mLogPath of
    Just logPath -> openPath logPath
    Nothing -> do
      let logPath = cacheDir wf ++ "/workflow.log"
      openPath logPath

-- | Open a path using the system open command.
-- Returns True on success, False on failure.
openPath :: String -> IO Bool
openPath path = do
  result <- try (callProcess "open" [path]) :: IO (Either IOException ())
  case result of
    Right _ -> pure True
    Left _  -> pure False
