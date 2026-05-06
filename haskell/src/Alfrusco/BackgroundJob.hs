{-# LANGUAGE OverloadedStrings #-}

-- | Background job management for Alfred workflows.
-- Provides the ability to run commands in the background and track their status.
module Alfrusco.BackgroundJob
  ( JobExecutionStatus (..)
  , BackgroundJobStatus (..)
  , createJobId
  , shellEscape
  , runInBackground
  ) where

import Data.Hashable (hash)
import Data.Text (Text)
import Data.Text qualified as Text
import Data.Time.Clock (NominalDiffTime, diffUTCTime, getCurrentTime)
import Numeric (showHex)
import System.Directory (createDirectoryIfMissing, doesFileExist, getModificationTime)
import System.Exit (ExitCode (..))
import System.FilePath ((</>))
import System.Process (CmdSpec (..), CreateProcess (..), StdStream (..), createProcess, getPid, readCreateProcessWithExitCode, shell)

import Alfrusco.Item (item, withSubtitle, withValid)
import Alfrusco.Workflow (Workflow, appendItems, cacheDir, setRerun)

-- | Status of a background job execution.
data JobExecutionStatus = JobSuccess | JobFailed | JobRunning | JobUnknown
  deriving (Show, Eq)

-- | Status indicating whether a job's results are fresh or stale.
data BackgroundJobStatus
  = Fresh NominalDiffTime
  | Stale (Maybe NominalDiffTime) NominalDiffTime
  deriving (Show)

-- | Create a filesystem-safe job ID by hashing the job name.
createJobId :: Text -> String
createJobId name =
  let h = abs (hash (Text.unpack name))
  in showHex (h :: Int) ""

-- | Escape a string for safe use in shell commands.
shellEscape :: String -> String
shellEscape [] = "''"
shellEscape s
  | all isSafe s = s
  | otherwise = "'" ++ concatMap escapeChar s ++ "'"
  where
    isSafe c = (c >= 'a' && c <= 'z')
            || (c >= 'A' && c <= 'Z')
            || (c >= '0' && c <= '9')
            || c == '-' || c == '_' || c == '.' || c == '/'
    escapeChar '\'' = "\"'\"'"
    escapeChar c = [c]

-- | Run a command in the background, adding a status item to the workflow
-- if the job is stale.
runInBackground :: Workflow -> Text -> Double -> CreateProcess -> IO ()
runInBackground wf jobName maxAge cmd = do
  let jobId = createJobId jobName
      dir = jobDirPath wf jobId
  createDirectoryIfMissing True dir

  staleness <- getStaleness dir
  let maxAgeNDT = realToFrac maxAge :: NominalDiffTime

  case staleness of
    Just s | s < maxAgeNDT -> do
      -- Check status
      status <- getJobStatus dir
      case status of
        JobSuccess -> pure ()  -- Job is fresh and successful
        _ -> doRun wf jobName dir cmd staleness  -- Failed, retry
    _ -> doRun wf jobName dir cmd staleness

-- | Internal: perform the actual run logic.
doRun :: Workflow -> Text -> FilePath -> CreateProcess -> Maybe NominalDiffTime -> IO ()
doRun wf jobName dir cmd staleness = do
  -- Check if already running
  running <- isJobRunning dir
  if running
    then do
      -- Already running, show status item
      let statusItem = withValid False
                     $ withSubtitle "Running in background..."
                     $ item ("Background Job '" <> jobName <> "'")
      appendItems wf [statusItem]
      setRerun wf 1.0
    else do
      -- Start the job
      startJob dir cmd
      let subtitle = maybe "First run, starting..." formatStaleness staleness
          statusItem = withValid False
                     $ withSubtitle subtitle
                     $ item ("Background Job '" <> jobName <> "'")
      appendItems wf [statusItem]
      setRerun wf 1.0

-- | Format a staleness duration for display.
formatStaleness :: NominalDiffTime -> Text
formatStaleness s =
  let secs = round s :: Int
  in Text.pack ("Last run " ++ show secs ++ "s ago, refreshing...")

-- | Get the job directory under the workflow cache.
jobDirPath :: Workflow -> String -> FilePath
jobDirPath wf jobId = cacheDir wf </> "jobs" </> jobId

-- | Check if a job is currently running by reading the PID from the PID file
-- and checking if that process is still alive.
isJobRunning :: FilePath -> IO Bool
isJobRunning dir = do
  let pidFile = dir </> "job.pid"
  exists <- doesFileExist pidFile
  if exists
    then do
      content <- readFile pidFile
      let pidStr = filter (/= '\n') content
      case reads pidStr :: [(Int, String)] of
        [(pid, "")] -> isProcessAlive pid
        _           -> pure False  -- Invalid PID file content, treat as not running
    else pure False

-- | Check if a process with the given PID is still alive.
-- Sends signal 0 which checks existence without actually signaling.
isProcessAlive :: Int -> IO Bool
isProcessAlive pid = do
  -- Use kill -0 to check if process exists
  (exitCode, _, _) <- readCreateProcessWithExitCode
    (shell ("kill -0 " ++ show pid ++ " 2>/dev/null")) ""
  pure (exitCode == ExitSuccess)

-- | Get the staleness (time since last run) of a job.
getStaleness :: FilePath -> IO (Maybe NominalDiffTime)
getStaleness dir = do
  let lastRunFile = dir </> "job.last_run"
  exists <- doesFileExist lastRunFile
  if exists
    then do
      mtime <- getModificationTime lastRunFile
      now <- getCurrentTime
      pure (Just (diffUTCTime now mtime))
    else pure Nothing

-- | Get the execution status of a job from its status file.
getJobStatus :: FilePath -> IO JobExecutionStatus
getJobStatus dir = do
  let statusFile = dir </> "job.status"
  exists <- doesFileExist statusFile
  if exists
    then do
      content <- readFile statusFile
      case filter (/= '\n') content of
        "success" -> pure JobSuccess
        "failed"  -> pure JobFailed
        "running" -> pure JobRunning
        _         -> pure JobUnknown
    else pure JobUnknown

-- | Start a background job by spawning a bash process.
startJob :: FilePath -> CreateProcess -> IO ()
startJob dir cmd = do
  let pidFile = dir </> "job.pid"
      statusFile = dir </> "job.status"
      lastRunFile = dir </> "job.last_run"
      logsFile = dir </> "job.logs"
      -- Extract the command string
      cmdString = case cmdspec cmd of
        ShellCommand s -> s
        RawCommand prog args -> unwords (shellEscape prog : map shellEscape args)
      -- Create a bash script that runs the command, writes status, updates last_run
      bashScript = cmdString
                   ++ " > " ++ shellEscape logsFile ++ " 2>&1; "
                   ++ "if [ $? -eq 0 ]; then echo success > " ++ shellEscape statusFile
                   ++ "; else echo failed > " ++ shellEscape statusFile ++ "; fi; "
                   ++ "touch " ++ shellEscape lastRunFile ++ "; "
                   ++ "rm -f " ++ shellEscape pidFile

  -- Write status as running
  writeFile statusFile "running"

  -- Spawn the background process
  (_, _, _, ph) <- createProcess (shell bashScript)
    { std_in = NoStream
    , std_out = NoStream
    , std_err = NoStream
    , create_group = True
    , delegate_ctlc = False
    }

  -- Get the actual PID and write it to the PID file
  mPid <- getPid ph
  case mPid of
    Just pid -> writeFile pidFile (show pid)
    Nothing  -> writeFile pidFile "0"  -- Fallback; isJobRunning will detect dead process
