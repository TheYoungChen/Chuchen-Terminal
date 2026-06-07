import assert from 'node:assert/strict'

describe('Chuchen Terminal desktop smoke', () => {
  it('opens the workspace shell', async () => {
    await browser.pause(5000)

    const handles = await browser.getWindowHandles()
    const currentHandle = await browser.getWindowHandle()
    const tree = await browser.browsingContextGetTree({})

    console.log('window handles =>', handles)
    console.log('current handle =>', currentHandle)
    console.log('context tree =>', JSON.stringify(tree, null, 2))

    assert.ok(handles.length >= 1)
  })
})
