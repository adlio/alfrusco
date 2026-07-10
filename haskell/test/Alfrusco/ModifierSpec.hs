{-# LANGUAGE OverloadedStrings #-}

module Alfrusco.ModifierSpec (spec) where

import Data.Aeson (encode, object, (.=))
import Data.Aeson qualified as Aeson
import Data.ByteString.Lazy.Char8 qualified as LBS
import Test.Hspec

import Alfrusco.Modifier

spec :: Spec
spec = describe "Alfrusco.Modifier" $ do
  describe "key display" $ do
    it "individual keys to text" $ do
      keyToText Cmd `shouldBe` "cmd"
      keyToText Ctrl `shouldBe` "ctrl"
      keyToText Alt `shouldBe` "alt"
      keyToText Shift `shouldBe` "shift"
      keyToText Fn `shouldBe` "fn"

  describe "combo keys" $ do
    it "cmd+shift" $ do
      keysToText [Cmd, Shift] `shouldBe` "cmd+shift"

    it "ctrl+fn" $ do
      keysToText [Ctrl, Fn] `shouldBe` "ctrl+fn"

    it "all keys combined" $ do
      keysToText [Cmd, Ctrl, Alt, Shift, Fn] `shouldBe` "cmd+ctrl+alt+shift+fn"

  describe "modifier JSON serialization" $ do
    it "serializes modifier with subtitle only" $ do
      let m = withModSubtitle "Hold Cmd" $ modifier Cmd
          decoded = Aeson.decode (encode m) :: Maybe Aeson.Value
          expected = Just $ object ["subtitle" .= ("Hold Cmd" :: String)]
      decoded `shouldBe` expected

    it "omits Nothing fields" $ do
      let m = modifier Cmd
          decoded = Aeson.decode (encode m) :: Maybe Aeson.Value
          expected = Just $ object []
      decoded `shouldBe` expected

    it "does not include keys field in JSON" $ do
      let m = withModSubtitle "test" $ modifierCombo [Cmd, Shift]
          json = LBS.unpack (encode m)
      -- The JSON should only have subtitle, not keys
      json `shouldNotContain` "\"keys\""
      json `shouldContain` "\"subtitle\""

  describe "modifier variables" $ do
    it "withModVar adds variables" $ do
      let m = withModVar "key1" "val1"
            $ withModVar "key2" "val2"
            $ modifier Cmd
          decoded = Aeson.decode (encode m) :: Maybe Aeson.Value
          expected = Just $ object
            [ "variables" .= object
                [ "key1" .= ("val1" :: String)
                , "key2" .= ("val2" :: String)
                ]
            ]
      decoded `shouldBe` expected

  describe "modifier arg" $ do
    it "withModArg sets arg" $ do
      let m = withModArg "my-arg" $ modifier Alt
          decoded = Aeson.decode (encode m) :: Maybe Aeson.Value
          expected = Just $ object ["arg" .= ("my-arg" :: String)]
      decoded `shouldBe` expected
