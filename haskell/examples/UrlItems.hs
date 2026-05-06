{-# LANGUAGE FunctionalDependencies #-}
{-# LANGUAGE OverloadedStrings #-}

-- | Equivalent to Rust examples/url_items.rs
-- Outputs URLItems with cache settings
module Main (main) where

import System.IO (stdout)

import Alfrusco

data UrlItemsWorkflow = UrlItemsWorkflow

instance Runnable UrlItemsWorkflow AlfruscoError where
  run _ wf = do
    setSkipKnowledge wf True
    setCache wf 60 True
    appendItems wf
      [ toItem (urlItem "DuckDuckGo" "https://www.duckduckgo.com")
      , toItem (urlItem "Google" "https://www.google.com")
      ]
    pure (Right ())

main :: IO ()
main = execute AlfredEnvProvider UrlItemsWorkflow stdout
