#!/usr/bin/env node
import { spawnSync } from 'node:child_process'
import os from 'node:os'

const mode = process.argv[2] || 'dev'
const isLinux = os.platform() === 'linux'

function run(cmd, args, opts = {}) {
  return spawnSync(cmd, args, {
    stdio: 'inherit',
    env: { ...process.env, CI: 'false' },
    ...opts,
  })
}

function hasCommand(cmd) {
  const result = spawnSync('bash', ['-lc', `command -v ${cmd}`], { stdio: 'ignore' })
  return result.status === 0
}

function hasPkg(pkg) {
  const result = spawnSync('pkg-config', ['--exists', pkg], { stdio: 'ignore' })
  return result.status === 0
}

if (isLinux) {
  const missing = []

  if (!hasCommand('pkg-config')) {
    missing.push('pkg-config')
  } else {
    for (const pkg of ['gtk+-3.0', 'webkit2gtk-4.1']) {
      if (!hasPkg(pkg)) missing.push(pkg)
    }
  }

  if (missing.length > 0) {
    console.error('\n[Tauri preflight] 缺少 Linux 原生依赖：')
    for (const item of missing) console.error(`- ${item}`)
    console.error('\n先安装这些系统包再跑：')
    console.error('sudo apt update && sudo apt install -y pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev')
    process.exit(1)
  }
}

const tauriArgs = mode === 'build' ? ['tauri', 'build'] : ['tauri', 'dev']
const result = run('npx', tauriArgs)
process.exit(result.status ?? 1)
