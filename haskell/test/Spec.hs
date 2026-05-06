module Main (main) where

import Test.Hspec
import Alfrusco

main :: IO ()
main = hspec $ do
  describe "Alfrusco" $ do
    it "exports a placeholder value" $ do
      placeholder `shouldBe` "alfrusco"
