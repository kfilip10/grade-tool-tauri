import { listen } from '@tauri-apps/api/event'
import { writable, type Writable } from 'svelte/store'

// Define types for better type safety
type ShinyStatus = 'idle' | 'running' | 'stopped' | 'error' | string;

// Create Svelte stores with proper typing
export const shinyStatus: Writable<ShinyStatus> = writable('idle')
export const shinyUrl = writable('')
export const shinyError = writable<string | null>(null)

// Function to initialize listeners
export function initShinyListeners() {
  listen('shiny-status', (event) => {
    console.log('Shiny status:', event.payload)
    shinyStatus.set(event.payload as string)
  })
  
  listen('shiny-started', (event) => {
    console.log('Shiny started at:', event.payload)
    shinyUrl.set(event.payload as string)
    shinyStatus.set('running')
  })
  
  listen('shiny-stopped', () => {
    console.log('Shiny stopped')
    shinyStatus.set('stopped')
    shinyUrl.set('')
  })
  
  listen('shiny-error', (event) => {
    console.error('Shiny error:', event.payload)
    shinyError.set(event.payload as string)
    shinyStatus.set('error')
  })
}