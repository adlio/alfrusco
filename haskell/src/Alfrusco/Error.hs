{-# LANGUAGE OverloadedStrings #-}

-- | Error types and the IsWorkflowError typeclass for Alfred workflows.
module Alfrusco.Error
  ( AlfruscoError (..)
  , IsWorkflowError (..)
  ) where

import Control.Exception (Exception, IOException)
import Data.Text (Text)
import Data.Text qualified as Text

import Alfrusco.Item (Item, item, withSubtitle)

-- | The main error type for the Alfrusco library.
data AlfruscoError
  = IoError IOException
  | ConfigError Text
  | ClipboardError Text
  | LoggingError Text
  | WorkflowError Text
  | MissingEnvVar Text
  deriving (Show)

instance Exception AlfruscoError

-- | Typeclass for errors that can be displayed as Alfred items.
class (Show e) => IsWorkflowError e where
  -- | Convert an error into an Alfred Item for display to the user.
  errorItem :: e -> Item
  errorItem e = item (Text.pack ("An error occurred: " ++ show e))

instance IsWorkflowError AlfruscoError where
  errorItem (IoError ex) =
    withSubtitle "IOException" $ item (Text.pack ("Error: " ++ show ex))
  errorItem (ConfigError msg) =
    withSubtitle "ConfigError" $ item ("Error: " <> msg)
  errorItem (ClipboardError msg) =
    withSubtitle "ClipboardError" $ item ("Error: " <> msg)
  errorItem (LoggingError msg) =
    withSubtitle "LoggingError" $ item ("Error: " <> msg)
  errorItem (WorkflowError msg) =
    withSubtitle "WorkflowError" $ item ("Error: " <> msg)
  errorItem (MissingEnvVar var) =
    withSubtitle "MissingEnvVar" $ item ("Error: Missing environment variable: " <> var)
