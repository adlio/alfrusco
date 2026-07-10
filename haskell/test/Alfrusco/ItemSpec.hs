{-# LANGUAGE OverloadedStrings #-}

module Alfrusco.ItemSpec (spec) where

import Data.Aeson (encode, object, (.=))
import Data.Aeson qualified as Aeson
import Data.Map.Strict qualified as Map
import Test.Hspec

import Alfrusco.Item
import Alfrusco.Modifier (Key (..), modifier, withModSubtitle)

spec :: Spec
spec = describe "Alfrusco.Item" $ do
  describe "item creation" $ do
    it "creates item with just title producing {\"title\":\"Test\"}" $ do
      let i = item "Test"
          json = encode i
      json `shouldBe` encode (object ["title" .= ("Test" :: String)])

    it "creates item with subtitle, arg, uid, valid" $ do
      let i = withValid True
            $ withUid "my-uid"
            $ withArg "my-arg"
            $ withSubtitle "My Subtitle"
            $ item "My Title"
          decoded = Aeson.decode (encode i) :: Maybe Aeson.Value
          expected = Just $ object
            [ "title" .= ("My Title" :: String)
            , "subtitle" .= ("My Subtitle" :: String)
            , "arg" .= ("my-arg" :: String)
            , "uid" .= ("my-uid" :: String)
            , "valid" .= True
            ]
      decoded `shouldBe` expected

  describe "variables" $ do
    it "withVar adds to variables" $ do
      let i = withVar "key1" "val1"
            $ withVar "key2" "val2"
            $ item "Test"
      itemVariables i `shouldBe` Map.fromList [("key1", "val1"), ("key2", "val2")]

    it "withUnsetVar removes a variable" $ do
      let i = withUnsetVar "key1"
            $ withVar "key1" "val1"
            $ withVar "key2" "val2"
            $ item "Test"
      itemVariables i `shouldBe` Map.fromList [("key2", "val2")]

    it "empty variables map is omitted from JSON" $ do
      let i = item "Test"
          json = encode i
      json `shouldBe` encode (object ["title" .= ("Test" :: String)])

  describe "text field" $ do
    it "withCopyText sets copy text" $ do
      let i = withCopyText "copied" $ item "Test"
          decoded = Aeson.decode (encode i) :: Maybe Aeson.Value
          expected = Just $ object
            [ "title" .= ("Test" :: String)
            , "text" .= object ["copy" .= ("copied" :: String)]
            ]
      decoded `shouldBe` expected

    it "withLargeTypeText sets large type text" $ do
      let i = withLargeTypeText "large" $ item "Test"
          decoded = Aeson.decode (encode i) :: Maybe Aeson.Value
          expected = Just $ object
            [ "title" .= ("Test" :: String)
            , "text" .= object ["largetype" .= ("large" :: String)]
            ]
      decoded `shouldBe` expected

  describe "icon" $ do
    it "withIconFromImage sets icon path" $ do
      let i = withIconFromImage "icon.png" $ item "Test"
          decoded = Aeson.decode (encode i) :: Maybe Aeson.Value
          expected = Just $ object
            [ "title" .= ("Test" :: String)
            , "icon" .= object ["path" .= ("icon.png" :: String)]
            ]
      decoded `shouldBe` expected

    it "withIconForFiletype sets icon type and path" $ do
      let i = withIconForFiletype "public.folder" $ item "Test"
          decoded = Aeson.decode (encode i) :: Maybe Aeson.Value
          expected = Just $ object
            [ "title" .= ("Test" :: String)
            , "icon" .= object ["type" .= ("filetype" :: String), "path" .= ("public.folder" :: String)]
            ]
      decoded `shouldBe` expected

  describe "boost and sticky" $ do
    it "boost is NOT in serialized JSON" $ do
      let i = withBoost 100 $ item "Test"
          json = encode i
      json `shouldBe` encode (object ["title" .= ("Test" :: String)])

    it "sticky is NOT in serialized JSON" $ do
      let i = withSticky True $ item "Test"
          json = encode i
      json `shouldBe` encode (object ["title" .= ("Test" :: String)])

  describe "modifiers" $ do
    it "withModifier adds to mods map" $ do
      let m = withModSubtitle "Hold Cmd" $ modifier Cmd
          i = withModifier m $ item "Test"
      Map.member "cmd" (itemModifiers i) `shouldBe` True

  describe "args" $ do
    it "withArgs produces array arg" $ do
      let i = withArgs ["one", "two", "three"] $ item "Test"
          decoded = Aeson.decode (encode i) :: Maybe Aeson.Value
          expected = Just $ object
            [ "title" .= ("Test" :: String)
            , "arg" .= (["one", "two", "three"] :: [String])
            ]
      decoded `shouldBe` expected
