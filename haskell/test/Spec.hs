{-# LANGUAGE OverloadedStrings #-}

module Main (main) where

import Test.Hspec

import qualified Alfrusco.ClipboardSpec
import qualified Alfrusco.ConfigSpec
import qualified Alfrusco.ItemSpec
import qualified Alfrusco.ModifierSpec
import qualified Alfrusco.ResponseSpec
import qualified Alfrusco.SortAndFilterSpec
import qualified Alfrusco.URLItemSpec
import qualified Alfrusco.WorkflowSpec

main :: IO ()
main = hspec $ do
  Alfrusco.ItemSpec.spec
  Alfrusco.ResponseSpec.spec
  Alfrusco.ModifierSpec.spec
  Alfrusco.SortAndFilterSpec.spec
  Alfrusco.URLItemSpec.spec
  Alfrusco.ConfigSpec.spec
  Alfrusco.ClipboardSpec.spec
  Alfrusco.WorkflowSpec.spec
