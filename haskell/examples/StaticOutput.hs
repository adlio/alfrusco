{-# LANGUAGE FunctionalDependencies #-}
{-# LANGUAGE OverloadedStrings #-}

-- | Equivalent to Rust examples/static_output.rs
-- Outputs 3 items with skipknowledge=true
module Main (main) where

import System.IO (stdout)

import Alfrusco

data StaticOutputWorkflow = StaticOutputWorkflow

instance Runnable StaticOutputWorkflow AlfruscoError where
  run _ wf = do
    setSkipKnowledge wf True
    appendItems wf
      [ withSubtitle "First Subtitle" $ item "First Option"
      , withSubtitle "Second Subtitle" $ item "Option 2"
      , withSubtitle "3" $ item "Three"
      ]
    pure (Right ())

main :: IO ()
main = execute AlfredEnvProvider StaticOutputWorkflow stdout
