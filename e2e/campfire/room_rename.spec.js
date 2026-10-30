import { test, expect } from '@playwright/test'
import { signIn } from './helpers.js'

// RENAMING A ROOM POSTS TO THE ROOM'S OWN CLASS ROUTE.
//
// Campfire's rooms are STI rows (`Rooms::Open`, `Rooms::Closed`,
// `Rooms::Direct`) and its shared `rooms/layouts/_form` calls
// `form_with model: room`. Rails' `polymorphic_path` names the action
// from the record's class, so editing the first-run room — a
// `Rooms::Open` — PATCHes `/rooms/opens/:id`, which
// `Rooms::OpensController#update` serves. `RoomsController` has no
// `update`, so a form that names the base route instead (`/rooms/:id`)
// saves nothing.

test('renaming an open room saves through its own route', async ({ page }) => {
  await signIn(page)

  // The signed-in root redirects to the last room the user visited —
  // after first run, the open room it created.
  await page.goto('/')
  await expect(page).toHaveURL(/\/rooms\/\d+/)
  const id = page.url().match(/\/rooms\/(\d+)/)[1]

  await page.goto(`/rooms/opens/${id}/edit`)
  const form = page.locator(`form:has(#room_name)`)
  await expect(form).toHaveAttribute('action', `/rooms/opens/${id}`)

  const name = `Renamed ${Date.now()}`
  await page.locator('#room_name').fill(name)
  const [response] = await Promise.all([
    page.waitForResponse(res => res.request().method() === 'POST' && res.url().includes(`/rooms/opens/${id}`)),
    form.locator('button[type="submit"]').click(),
  ])
  expect(response.status()).toBeLessThan(400)

  await page.goto(`/rooms/opens/${id}/edit`)
  await expect(page.locator('#room_name')).toHaveValue(name)
})
