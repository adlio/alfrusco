{-# LANGUAGE FunctionalDependencies #-}
{-# LANGUAGE OverloadedStrings #-}

module Alfrusco.WorkflowSpec (spec) where

import Data.Aeson (Value (..))
import Data.Aeson qualified as Aeson
import Data.Aeson.KeyMap qualified as KM
import Data.ByteString.Lazy qualified as LBS
import Data.IORef (readIORef)
import Data.Sequence qualified as Seq
import Data.Text qualified as Text
import Data.Vector qualified as V
import System.IO (IOMode (..), hClose, openBinaryFile)
import System.IO.Temp (withSystemTempDirectory)
import Test.Hspec

import Alfrusco.Config (ConfigProvider (..), TestingProvider (..))
import Alfrusco.Error (AlfruscoError (..), IsWorkflowError (..))
import Alfrusco.Item (Item (..), item, withSubtitle)
import Alfrusco.Response (Response (..))
import Alfrusco.Runnable (Runnable (..), execute)
import Alfrusco.Workflow

-- | A simple test runnable that appends items to the workflow.
newtype TestRunnable = TestRunnable [Item]

instance Runnable TestRunnable AlfruscoError where
  run (TestRunnable items) wf = do
    appendItems wf items
    pure (Right ())

-- | A test runnable that returns an error.
data FailingRunnable = FailingRunnable

instance Runnable FailingRunnable AlfruscoError where
  run FailingRunnable _wf = pure (Left (WorkflowError "something went wrong"))

spec :: Spec
spec = describe "Alfrusco.Workflow" $ do
  describe "appendItems (Seq-based)" $ do
    it "appends items to an empty workflow" $ do
      withSystemTempDirectory "alfrusco-test" $ \dir -> do
        let provider = TestingProvider dir
        Right config <- getConfig provider
        wf <- newWorkflow config
        appendItems wf [item "First", item "Second"]
        response <- readIORef (wfResponse wf)
        Seq.length (responseItems response) `shouldBe` 2

    it "appends multiple batches and preserves order" $ do
      withSystemTempDirectory "alfrusco-test" $ \dir -> do
        let provider = TestingProvider dir
        Right config <- getConfig provider
        wf <- newWorkflow config
        appendItems wf [item "A", item "B"]
        appendItems wf [item "C", item "D"]
        response <- readIORef (wfResponse wf)
        let titles = fmap itemTitle (responseItems response)
        toList titles `shouldBe` ["A", "B", "C", "D"]

  describe "prependItem" $ do
    it "prepends an item before existing items" $ do
      withSystemTempDirectory "alfrusco-test" $ \dir -> do
        let provider = TestingProvider dir
        Right config <- getConfig provider
        wf <- newWorkflow config
        appendItems wf [item "Second", item "Third"]
        prependItem wf (item "First")
        response <- readIORef (wfResponse wf)
        let titles = fmap itemTitle (responseItems response)
        toList titles `shouldBe` ["First", "Second", "Third"]

  describe "execute integration" $ do
    it "produces valid JSON output with items" $ do
      withSystemTempDirectory "alfrusco-test" $ \dir -> do
        let provider = TestingProvider dir
            runnable = TestRunnable [item "Hello", withSubtitle "world" $ item "Test"]
            outFile = dir ++ "/output.json"
        h <- openBinaryFile outFile WriteMode
        execute provider runnable h
        hClose h
        jsonBytes <- LBS.readFile outFile
        case Aeson.decode jsonBytes :: Maybe Value of
          Nothing -> expectationFailure "Output is not valid JSON"
          Just (Object obj) -> do
            case KM.lookup "items" obj of
              Just (Array arr) -> V.length arr `shouldBe` 2
              _ -> expectationFailure "Expected 'items' array in response"
          Just _ -> expectationFailure "Expected JSON object"

    it "prepends error item on runnable failure" $ do
      withSystemTempDirectory "alfrusco-test" $ \dir -> do
        let provider = TestingProvider dir
            outFile = dir ++ "/output.json"
        h <- openBinaryFile outFile WriteMode
        execute provider FailingRunnable h
        hClose h
        jsonBytes <- LBS.readFile outFile
        case Aeson.decode jsonBytes :: Maybe Value of
          Nothing -> expectationFailure "Output is not valid JSON"
          Just (Object obj) -> do
            case KM.lookup "items" obj of
              Just (Array arr) -> do
                V.length arr `shouldSatisfy` (>= 1)
                -- The first item should be the error item
                case V.head arr of
                  Object itemObj ->
                    case KM.lookup "title" itemObj of
                      Just (String t) -> t `shouldSatisfy` Text.isInfixOf "something went wrong"
                      _ -> expectationFailure "Error item should have a title"
                  _ -> expectationFailure "Expected item object"
              _ -> expectationFailure "Expected 'items' array in response"
          Just _ -> expectationFailure "Expected JSON object"

-- | Convert Seq to list.
toList :: Seq.Seq a -> [a]
toList = foldr (:) []
