import { invoke } from '@tauri-apps/api/core';
import { writable } from 'svelte/store';
import { stopBrowserMonitoring } from './windowManager';

// Status tracking for the Shiny process
export const shinyStatus = writable<'idle' | 'starting' | 'running' | 'stopping' | 'stopped' | 'error'>('idle');
export const shinyUrl = writable<string | null>(null);
export const shinyError = writable<string | null>(null);

/**
 * Launch the Shiny application
 */
export async function launchShinyApp(): Promise<string> {
  try {
    shinyStatus.set('starting');
    shinyError.set(null);
    
    // Start R Shiny app
    const url = await invoke<string>('start_r_shiny');
    
    // Update stores
    shinyUrl.set(url);
    shinyStatus.set('running');
    
    return url;
  } catch (error) {
    console.error('Failed to start Shiny app:', error);
    shinyError.set(String(error));
    shinyStatus.set('error');
    throw error;
  }
}

/**
 * Stop the running Shiny application
 */
export async function stopShinyApp(): Promise<void> {
  // Stop browser monitoring
  stopBrowserMonitoring();
  
  try {
    shinyStatus.set('stopping');
    await invoke<void>('stop_r_shiny');
    shinyStatus.set('stopped');
    shinyUrl.set(null);
  } catch (error) {
    console.error('Failed to stop Shiny app:', error);
    shinyError.set(String(error));
    shinyStatus.set('error');
    throw error;
  }
}