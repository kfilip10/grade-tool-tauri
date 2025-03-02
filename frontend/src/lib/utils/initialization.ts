import { invoke } from '@tauri-apps/api/core';
import { writable } from 'svelte/store';
import { confirm } from '@tauri-apps/plugin-dialog';
import { updateProgressVisible, checkForUpdates } from './updater';
import { shinyStatus, shinyUrl, shinyError } from './shinyListener';
import { openBrowserWindow } from './windowManager'; // Update import

// App initialization stages
export type InitStatus = 
  | 'starting' 
  | 'checking-updates'
  | 'downloading-update'
  | 'update-ready'
  | 'update-failed'
  | 'launching-shiny'
  | 'shiny-ready'
  | 'error';

  // Create step status types (for tracking progress with stepindicator.svelte)
export type StepStatus = 'pending' | 'loading' | 'completed' | 'error';

// Add stores for tracking individual step statuses
export const updateCheckStatus = writable<StepStatus>('pending');
export const shinyLaunchStatus = writable<StepStatus>('pending');


// Store for tracking init process
export const initStatus = writable<InitStatus>('starting');
export const initMessage = writable<string>('Starting application...');
export const initError = writable<string | null>(null);

/**
 * Main application initialization flow
 */
export async function initializeApp(): Promise<void> {
  try {
    // 1. Check for updates
    initStatus.set('checking-updates');
    initMessage.set('Checking for updates...');
    updateCheckStatus.set('loading');

    try {
      // Use the existing checkForUpdates function
      await checkForUpdates();
      
      // If we're still here, either no update was available
      // or the user chose to skip the update
      console.log('Continuing initialization after update check');
      updateCheckStatus.set('completed');

    } catch (error) {
      // If checkForUpdates throws an error, log it and continue
      console.error('Update check failed:', error);
      updateCheckStatus.set('error');

      // This is non-critical, so we can continue with initialization
    }    


    // 2. Launch Shiny app
    initStatus.set('launching-shiny');
    initMessage.set('Launching Shiny application...');
    shinyLaunchStatus.set('loading');

    try {
      const shinyAppUrl = await invoke<string>('start_r_shiny');
      
      // Set stores
      shinyUrl.set(shinyAppUrl);
      shinyStatus.set('running');
      shinyLaunchStatus.set('completed');
      // Open the Shiny URL in the default browser
      await openBrowserWindow(shinyAppUrl);
      
      // Update the status to show it's running
      initStatus.set('shiny-ready');
      initMessage.set('Shiny application is running in your browser');
      
    } catch (error) {
      console.error('Failed to start Shiny:', error);
      initStatus.set('error');
      initMessage.set(`Failed to start Shiny: ${error}`);
      initError.set(String(error));
      shinyStatus.set('error');
      shinyError.set(String(error));
      shinyLaunchStatus.set('error');
    }
  } catch (error) {
    console.error('Initialization error:', error);
    initStatus.set('error');
    initMessage.set(`Initialization error: ${error}`);
    initError.set(String(error));
  }
}

// Reset all step statuses to pending
export function resetStepStatuses(): void {
  updateCheckStatus.set('pending');
  shinyLaunchStatus.set('pending');
}