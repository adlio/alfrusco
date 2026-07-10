{-# LANGUAGE OverloadedStrings #-}

-- | Configuration types and providers for Alfred workflows.
module Alfrusco.Config
  ( WorkflowConfig (..)
  , ConfigProvider (..)
  , AlfredEnvProvider (..)
  , TestingProvider (..)
  ) where

import Data.Text (Text)
import Data.Text qualified as Text
import System.Environment (lookupEnv)

import Alfrusco.Error (AlfruscoError (..))

-- | Holds the configuration values for the current workflow.
-- In a real-world scenario, these are read from environment variables set by Alfred.
data WorkflowConfig = WorkflowConfig
  { wcBundleId :: Text
  , wcCache :: FilePath
  , wcData :: FilePath
  , wcVersion :: Text
  , wcVersionBuild :: Text
  , wcName :: Text
  , wcWorkflowVersion :: Maybe Text
  , wcPreferences :: Maybe Text
  , wcPreferencesLocalHash :: Maybe Text
  , wcTheme :: Maybe Text
  , wcThemeBackground :: Maybe Text
  , wcThemeSelectionBackground :: Maybe Text
  , wcThemeSubtext :: Maybe Text
  , wcWorkflowDescription :: Maybe Text
  , wcWorkflowUid :: Maybe Text
  , wcWorkflowKeyword :: Maybe Text
  , wcDebug :: Bool
  }
  deriving (Show, Eq)

-- | Typeclass for providing workflow configuration.
class ConfigProvider p where
  getConfig :: p -> IO (Either AlfruscoError WorkflowConfig)

-- | Reads workflow configuration from Alfred environment variables.
data AlfredEnvProvider = AlfredEnvProvider

-- | Helper to get a required environment variable.
requireEnv :: String -> IO (Either AlfruscoError Text)
requireEnv var = do
  val <- lookupEnv var
  case val of
    Nothing -> pure $ Left $ MissingEnvVar (Text.pack ("Missing required environment variable: " ++ var))
    Just v -> pure $ Right (Text.pack v)

-- | Helper to get a required FilePath environment variable.
requireEnvPath :: String -> IO (Either AlfruscoError FilePath)
requireEnvPath var = do
  val <- lookupEnv var
  case val of
    Nothing -> pure $ Left $ MissingEnvVar (Text.pack ("Missing required environment variable: " ++ var))
    Just v -> pure $ Right v

-- | Helper to get an optional environment variable.
optionalEnv :: String -> IO (Maybe Text)
optionalEnv var = fmap (fmap Text.pack) (lookupEnv var)

instance ConfigProvider AlfredEnvProvider where
  getConfig _ = do
    eBundleId <- requireEnv "alfred_workflow_bundleid"
    eCache <- requireEnvPath "alfred_workflow_cache"
    eData <- requireEnvPath "alfred_workflow_data"
    eVersion <- requireEnv "alfred_version"
    eVersionBuild <- requireEnv "alfred_version_build"
    eName <- requireEnv "alfred_workflow_name"

    case (eBundleId, eCache, eData, eVersion, eVersionBuild, eName) of
      (Right bundleId, Right cache, Right wfData, Right version, Right versionBuild, Right name) -> do
        workflowVersion <- optionalEnv "alfred_workflow_version"
        preferences <- optionalEnv "alfred_preferences"
        preferencesLocalHash <- optionalEnv "alfred_preferences_localhash"
        theme <- optionalEnv "alfred_theme"
        themeBackground <- optionalEnv "alfred_theme_background"
        themeSelectionBackground <- optionalEnv "alfred_theme_selection_background"
        themeSubtext <- optionalEnv "alfred_theme_subtext"
        workflowDescription <- optionalEnv "alfred_workflow_description"
        workflowUid <- optionalEnv "alfred_workflow_uid"
        workflowKeyword <- optionalEnv "alfred_workflow_keyword"
        debugStr <- lookupEnv "alfred_debug"
        let debug = case debugStr of
              Just "1" -> True
              Just "true" -> True
              Just "True" -> True
              _ -> False

        pure $ Right WorkflowConfig
          { wcBundleId = bundleId
          , wcCache = cache
          , wcData = wfData
          , wcVersion = version
          , wcVersionBuild = versionBuild
          , wcName = name
          , wcWorkflowVersion = workflowVersion
          , wcPreferences = preferences
          , wcPreferencesLocalHash = preferencesLocalHash
          , wcTheme = theme
          , wcThemeBackground = themeBackground
          , wcThemeSelectionBackground = themeSelectionBackground
          , wcThemeSubtext = themeSubtext
          , wcWorkflowDescription = workflowDescription
          , wcWorkflowUid = workflowUid
          , wcWorkflowKeyword = workflowKeyword
          , wcDebug = debug
          }
      (Left e, _, _, _, _, _) -> pure $ Left e
      (_, Left e, _, _, _, _) -> pure $ Left e
      (_, _, Left e, _, _, _) -> pure $ Left e
      (_, _, _, Left e, _, _) -> pure $ Left e
      (_, _, _, _, Left e, _) -> pure $ Left e
      (_, _, _, _, _, Left e) -> pure $ Left e

-- | Testing provider that returns hardcoded test values.
-- Uses subdirectories of the given path for cache and data.
newtype TestingProvider = TestingProvider FilePath

instance ConfigProvider TestingProvider where
  getConfig (TestingProvider basePath) = pure $ Right WorkflowConfig
    { wcBundleId = "com.alfredapp.googlesuggest"
    , wcCache = basePath ++ "/workflow_cache"
    , wcData = basePath ++ "/workflow_data"
    , wcVersion = "5.0"
    , wcVersionBuild = "2058"
    , wcName = "Test Workflow"
    , wcWorkflowVersion = Just "1.7"
    , wcPreferences = Just "/Users/Crayons/Dropbox/Alfred/Alfred.alfredpreferences"
    , wcPreferencesLocalHash = Just "adbd4f66bc3ae8493832af61a41ee609b20d8705"
    , wcTheme = Just "alfred.theme.yosemite"
    , wcThemeBackground = Just "rgba(255,255,255,0.98)"
    , wcThemeSelectionBackground = Just "rgba(255,255,255,0.98)"
    , wcThemeSubtext = Just "3"
    , wcWorkflowDescription = Just "The description of the workflow we use for testing"
    , wcWorkflowUid = Just "user.workflow.B0AC54EC-601C-479A-9428-01F9FD732959"
    , wcWorkflowKeyword = Nothing
    , wcDebug = True
    }
