{-# LANGUAGE OverloadedStrings #-}

module Main (main) where

import Test.Hspec
import Alfrusco

main :: IO ()
main = hspec $ do
  describe "Alfrusco" $ do
    it "can create an item with a title" $ do
      let i = item "Test"
      itemTitle i `shouldBe` "Test"
