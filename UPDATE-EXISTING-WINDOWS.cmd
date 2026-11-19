@echo off
cd /d "%~dp0"
node upgrade-existing.cjs
if errorlevel 1 (
  echo.
  echo The update did not finish. Read the message above.
  pause
)
