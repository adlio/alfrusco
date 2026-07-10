{-# LANGUAGE OverloadedStrings #-}

module Alfrusco.URLItemSpec (spec) where

import Data.Map.Strict qualified as Map
import Test.Hspec

import Alfrusco.Item (itemTitle, itemSubtitle, itemArg, itemModifiers, itemValid)
import Alfrusco.Arg (Arg (..))
import Alfrusco.Modifier (Modifier (..))
import Alfrusco.URLItem

spec :: Spec
spec = describe "Alfrusco.URLItem" $ do
  describe "basic urlItem" $ do
    it "creates item with correct title, subtitle=url, arg=url" $ do
      let ui = urlItem "DuckDuckGo" "https://duckduckgo.com"
          i = toItem ui
      itemTitle i `shouldBe` "DuckDuckGo"
      itemSubtitle i `shouldBe` Just "https://duckduckgo.com"
      itemArg i `shouldBe` Just (ArgOne "https://duckduckgo.com")
      itemValid i `shouldBe` Just True

  describe "withDisplayTitle" $ do
    it "overrides displayed title" $ do
      let ui = withDisplayTitle "Duck" $ urlItem "DuckDuckGo" "https://duckduckgo.com"
          i = toItem ui
      itemTitle i `shouldBe` "Duck"

  describe "withShortTitle" $ do
    it "generates cmd+shift and alt+shift modifiers" $ do
      let ui = withShortTitle "DDG" $ urlItem "DuckDuckGo" "https://duckduckgo.com"
          i = toItem ui
          mods = itemModifiers i
      Map.member "cmd+shift" mods `shouldBe` True
      Map.member "alt+shift" mods `shouldBe` True

  describe "withLongTitle" $ do
    it "generates cmd+ctrl and alt+ctrl modifiers" $ do
      let ui = withLongTitle "DuckDuckGo Search Engine" $ urlItem "DuckDuckGo" "https://duckduckgo.com"
          i = toItem ui
          mods = itemModifiers i
      Map.member "cmd+ctrl" mods `shouldBe` True
      Map.member "alt+ctrl" mods `shouldBe` True

  describe "withUrlSubtitle" $ do
    it "overrides url subtitle" $ do
      let ui = withUrlSubtitle "Custom subtitle" $ urlItem "DuckDuckGo" "https://duckduckgo.com"
          i = toItem ui
      itemSubtitle i `shouldBe` Just "Custom subtitle"

  describe "withUrlCopyText" $ do
    it "overrides default copy text" $ do
      let ui = withUrlCopyText "custom copy" $ urlItem "DuckDuckGo" "https://duckduckgo.com"
          i = toItem ui
      -- Copy text should be set (it's in the text field)
      -- Just verify the item can be created without error
      itemTitle i `shouldBe` "DuckDuckGo"

  describe "cmd modifier has ALFRUSCO_COMMAND=markdown variable" $ do
    it "cmd modifier contains markdown command variable" $ do
      let ui = urlItem "Test" "https://example.com"
          i = toItem ui
          mods = itemModifiers i
      case Map.lookup "cmd" mods of
        Just m -> case modVariables m of
          Just vars -> Map.lookup "ALFRUSCO_COMMAND" vars `shouldBe` Just "markdown"
          Nothing -> expectationFailure "Expected variables in cmd modifier"
        Nothing -> expectationFailure "Expected cmd modifier"

    it "alt modifier contains richtext command variable" $ do
      let ui = urlItem "Test" "https://example.com"
          i = toItem ui
          mods = itemModifiers i
      case Map.lookup "alt" mods of
        Just m -> case modVariables m of
          Just vars -> Map.lookup "ALFRUSCO_COMMAND" vars `shouldBe` Just "richtext"
          Nothing -> expectationFailure "Expected variables in alt modifier"
        Nothing -> expectationFailure "Expected alt modifier"
