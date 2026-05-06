{-# LANGUAGE OverloadedStrings #-}

-- | Logging initialization for Alfred workflows.
-- Sets up logging to both stderr and a file in the workflow cache directory.
module Alfrusco.Logging
  ( initLogging
  ) where

import System.Directory (createDirectoryIfMissing)
import System.FilePath ((</>), takeDirectory)
import System.Log.Handler (setFormatter)
import System.Log.Handler.Simple (fileHandler, streamHandler)
import System.Log.Formatter (simpleLogFormatter)
import System.Log.Logger (Priority (..), rootLoggerName, setHandlers, setLevel, updateGlobalLogger)
import System.IO (stderr)

import Alfrusco.Config (ConfigProvider (..), WorkflowConfig (..))
import Alfrusco.Error (AlfruscoError (..))

-- | Initialize logging to both stderr and a log file in the workflow cache directory.
-- Sets up the root logger with DEBUG level to file and INFO level to stderr.
initLogging :: ConfigProvider p => p -> IO (Either AlfruscoError ())
initLogging provider = do
  eConfig <- getConfig provider
  case eConfig of
    Left e -> pure (Left e)
    Right config -> do
      let logFilePath = wcCache config </> "workflow.log"
      -- Ensure the parent directory exists
      createDirectoryIfMissing True (takeDirectory logFilePath)

      -- Create file handler (DEBUG level)
      fh <- fileHandler logFilePath DEBUG
      let formattedFh = setFormatter fh (simpleLogFormatter "[$time $loggername $prio] $msg")

      -- Create stderr handler (INFO level)
      sh <- streamHandler stderr INFO
      let formattedSh = setFormatter sh (simpleLogFormatter "[$time $loggername $prio] $msg")

      -- Configure root logger
      updateGlobalLogger rootLoggerName (setLevel DEBUG . setHandlers [formattedFh, formattedSh])

      pure (Right ())
