{-# LANGUAGE OverloadedStrings #-}

module Alfrusco.ConfigSpec (spec) where

import Test.Hspec

import Alfrusco.Config

spec :: Spec
spec = describe "Alfrusco.Config" $ do
  describe "TestingProvider" $ do
    it "returns expected config values" $ do
      let provider = TestingProvider "/tmp/test"
      eConfig <- getConfig provider
      case eConfig of
        Left e -> expectationFailure ("Expected Right, got Left: " ++ show e)
        Right config -> do
          wcBundleId config `shouldBe` "com.alfredapp.googlesuggest"
          wcVersion config `shouldBe` "5.0"
          wcVersionBuild config `shouldBe` "2058"
          wcName config `shouldBe` "Test Workflow"
          wcCache config `shouldBe` "/tmp/test/workflow_cache"
          wcData config `shouldBe` "/tmp/test/workflow_data"
          wcWorkflowVersion config `shouldBe` Just "1.7"
          wcDebug config `shouldBe` True

  describe "AlfredEnvProvider" $ do
    it "returns error when required vars missing" $ do
      eConfig <- getConfig AlfredEnvProvider
      case eConfig of
        Left _ -> pure ()  -- Expected: missing env vars
        Right _ -> expectationFailure "Expected Left when Alfred env vars are missing"
