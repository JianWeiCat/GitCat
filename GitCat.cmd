@echo off
setlocal
chcp 65001 >nul
title GitCat

set "APP=%~dp0git-helper-ui\src-tauri\target\release\git-helper.exe"

if exist "%APP%" (
  start "GitCat" "%APP%"
  exit /b 0
)

cd /d "%~dp0git-helper-ui\src-tauri"
echo 未找到已构建的应用，正在以开发模式启动 GitCat…
echo 首次启动可能需要几秒钟，请保持此窗口打开。
echo.
cargo tauri dev --no-watch

if errorlevel 1 (
  echo.
  echo 启动失败。请确认已安装 Rust、Node.js 和项目依赖。
  pause
)
