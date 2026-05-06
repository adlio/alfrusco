{-# LANGUAGE OverloadedStrings #-}

-- | Workflow state management for Alfred workflows.
module Alfrusco.Workflow
  ( Workflow (..)
  , newWorkflow
  , appendItem
  , appendItems
  , prependItem
  , prependItems
  , setFilterKeyword
  , dataDir
  , cacheDir
  , setCache
  , setRerun
  , setSkipKnowledge
  , jobsDir
  ) where

import Data.IORef
import Data.Sequence qualified as Seq
import Data.Text (Text)
import System.Directory (createDirectoryIfMissing)
import System.FilePath ((</>))

import Alfrusco.Config (WorkflowConfig (..))
import Alfrusco.Item (Item)
import Alfrusco.Response (CacheSettings (..), Response (..), defaultResponse)

-- | Workflow represents an active execution of an Alfred workflow.
-- It maintains the state of the current Response and owns the Workflow
-- configuration information.
data Workflow = Workflow
  { wfConfig :: WorkflowConfig
  , wfResponse :: IORef Response
  , wfKeyword :: IORef (Maybe Text)
  , wfSortAndFilter :: IORef Bool
  }

-- | Create a new Workflow from a WorkflowConfig.
-- Creates the data and cache directories if they do not exist.
newWorkflow :: WorkflowConfig -> IO Workflow
newWorkflow config = do
  createDirectoryIfMissing True (wcData config)
  createDirectoryIfMissing True (wcCache config)
  responseRef <- newIORef defaultResponse
  keywordRef <- newIORef Nothing
  sortRef <- newIORef False
  pure Workflow
    { wfConfig = config
    , wfResponse = responseRef
    , wfKeyword = keywordRef
    , wfSortAndFilter = sortRef
    }

-- | Append a single item to the workflow response. O(1) amortized via Seq.
appendItem :: Workflow -> Item -> IO ()
appendItem wf i = modifyIORef' (wfResponse wf) $ \r ->
  r {responseItems = responseItems r Seq.|> i}

-- | Append multiple items to the workflow response. O(log(min(n,m))) via Seq.
appendItems :: Workflow -> [Item] -> IO ()
appendItems wf items = modifyIORef' (wfResponse wf) $ \r ->
  r {responseItems = responseItems r <> Seq.fromList items}

-- | Prepend a single item to the workflow response.
prependItem :: Workflow -> Item -> IO ()
prependItem wf i = modifyIORef' (wfResponse wf) $ \r ->
  r {responseItems = i Seq.<| responseItems r}

-- | Prepend multiple items to the workflow response.
prependItems :: Workflow -> [Item] -> IO ()
prependItems wf items = modifyIORef' (wfResponse wf) $ \r ->
  r {responseItems = Seq.fromList items <> responseItems r}

-- | Set the filter keyword and enable sort-and-filter.
setFilterKeyword :: Workflow -> Text -> IO ()
setFilterKeyword wf kw = do
  writeIORef (wfKeyword wf) (Just kw)
  writeIORef (wfSortAndFilter wf) True

-- | Get the data directory path.
dataDir :: Workflow -> FilePath
dataDir = wcData . wfConfig

-- | Get the cache directory path.
cacheDir :: Workflow -> FilePath
cacheDir = wcCache . wfConfig

-- | Set cache settings on the response.
-- The first parameter is the cache duration in seconds.
-- The second parameter controls loose reload behavior.
setCache :: Workflow -> Double -> Bool -> IO ()
setCache wf seconds looseReload = modifyIORef' (wfResponse wf) $ \r ->
  r {responseCache = Just CacheSettings
    { cacheSeconds = Just seconds
    , cacheLooseReload = Just looseReload
    }}

-- | Set the rerun interval in seconds.
setRerun :: Workflow -> Double -> IO ()
setRerun wf seconds = modifyIORef' (wfResponse wf) $ \r ->
  r {responseRerun = Just seconds}

-- | Set the skip knowledge flag.
setSkipKnowledge :: Workflow -> Bool -> IO ()
setSkipKnowledge wf skip = modifyIORef' (wfResponse wf) $ \r ->
  r {responseSkipKnowledge = Just skip}

-- | Get the jobs directory path (cacheDir / "jobs").
jobsDir :: Workflow -> FilePath
jobsDir wf = cacheDir wf </> "jobs"
