{-# LANGUAGE OverloadedStrings #-}

module Alfrusco.SortAndFilterSpec (spec) where

import Test.Hspec

import Alfrusco.Item (item, itemTitle, withBoost, withSticky, withSubtitle)
import Alfrusco.SortAndFilter (filterAndSortItems)

spec :: Spec
spec = describe "Alfrusco.SortAndFilter" $ do
  describe "basic matching" $ do
    it "items with matching subtitle are included" $ do
      let items = [ withSubtitle "fruit" $ item "Apple"
                  , withSubtitle "fruit" $ item "Banana"
                  , withSubtitle "vegetable" $ item "Carrot"
                  ]
          result = filterAndSortItems items "fruit"
          titles = map itemTitle result
      titles `shouldContain` ["Apple"]
      titles `shouldContain` ["Banana"]

    it "non-matching items are filtered out" $ do
      let items = [ withSubtitle "fruit" $ item "Apple"
                  , withSubtitle "vegetable" $ item "Carrot"
                  ]
          result = filterAndSortItems items "xyz"
      result `shouldBe` []

  describe "sticky items" $ do
    it "sticky items always come first regardless of query" $ do
      let items = [ withSubtitle "fruit" $ item "Apple"
                  , withSticky True $ withSubtitle "sticky" $ item "Pinned Item"
                  , withSubtitle "fruit" $ item "Banana"
                  ]
          result = filterAndSortItems items "fruit"
      -- Sticky item should be first even though it doesn't match
      case result of
        (first : _) -> itemTitle first `shouldBe` "Pinned Item"
        _ -> expectationFailure "Expected at least one item"

  describe "boost affects sort order" $ do
    it "higher boost comes first" $ do
      let items = [ withBoost 10 $ withSubtitle "fruit" $ item "Low Boost"
                  , withBoost 100 $ withSubtitle "fruit" $ item "High Boost"
                  , withBoost 50 $ withSubtitle "fruit" $ item "Medium Boost"
                  ]
          result = filterAndSortItems items "fruit"
          titles = map itemTitle result
      -- High boost should come before low boost
      head titles `shouldBe` "High Boost"

  describe "empty query" $ do
    it "empty query matches all items" $ do
      let items = [ item "One"
                  , item "Two"
                  , item "Three"
                  ]
          result = filterAndSortItems items ""
      length result `shouldBe` 3
