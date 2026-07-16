#!/usr/bin/env node

const totalLines = Math.max(1, Number.parseInt(process.argv[2] || '600', 10) || 600)
const delayMs = Math.max(0, Number.parseInt(process.argv[3] || '0', 10) || 0)
const label = process.argv[4] || 'CT_STRESS'

function write(line) {
  process.stdout.write(`${line}\n`)
}

async function sleep(ms) {
  if (!ms) return
  await new Promise((resolve) => setTimeout(resolve, ms))
}

async function main() {
  const startedAt = new Date().toISOString()
  write(`[${label}] start ${startedAt} lines=${totalLines} delayMs=${delayMs}`)

  for (let index = 1; index <= totalLines; index += 1) {
    write(`[${label}] line ${index}/${totalLines} ${new Date().toISOString()}`)
    if (delayMs) {
      await sleep(delayMs)
    }
  }

  write(`[${label}] done ${new Date().toISOString()}`)
}

main().catch((error) => {
  process.stderr.write(`[${label}] error ${error instanceof Error ? error.stack || error.message : String(error)}\n`)
  process.exitCode = 1
})
