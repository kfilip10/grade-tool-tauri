import { invoke } from '@tauri-apps/api/core';
import { relaunch } from '@tauri-apps/plugin-process';
import { message, ask } from '@tauri-apps/plugin-dialog';
import { get, writable } from 'svelte/store';

// Interface for the update info returned by Rust
interface UpdateInfo {
  available: boolean;
  version?: string;
  body?: string;
  date?: string;
  downloadUrl?: string;
}

// Interface for the progress info
interface UpdateProgress {
  downloading: boolean;
  percent: number;
  downloaded: number;
  total?: number;
  complete: boolean;
  error?: string;
}

// Create a store for managing the progress dialog visibility
export const updateProgressVisible = writable(false);
export const updateComplete = writable(false);
export const updateError = writable<string | null>(null);

export async function checkForUpdates() {
  try {
    // Check for updates using the Rust command
    const response = await invoke<string>('check_for_updates');
    const updateInfo: UpdateInfo = JSON.parse(response);
    
    console.log('Update check result:', updateInfo);
    
    if (!updateInfo.available) {
      console.log('No update available');
      //await message('You are running the latest version.', {
      //  title: 'No Update Available'
      //});
      return;
    }
    
    // Show confirmation dialog
    const shouldUpdate = await ask(
      `A new version (${updateInfo.version}) is available.\n\n` +
      `Release notes:\n${updateInfo.body || 'No release notes'}\n\n` +
      `Published on: ${updateInfo.date || 'Unknown date'}\n\n` +
      'Would you like to update now?',
      {
        title: 'Update Available',
        okLabel: 'Yes, update now',
        cancelLabel: 'No, remind me later'
      }
    );
    
    if (shouldUpdate) {
      // Reset state
      updateComplete.set(false);
      updateError.set(null);
      
      try {
        // Show progress dialog by setting the store value
        updateProgressVisible.set(true);
        
        // Start download and installation in Rust
        await invoke('download_and_install_update');
        
        // Wait until update is complete or has error
        await new Promise<void>((resolve, reject) => {
          const unsubComplete = updateComplete.subscribe(value => {
            if (value) {
              unsubComplete();
              unsubError();
              resolve();
            }
          });
          
          const unsubError = updateError.subscribe(err => {
            if (err) {
              unsubComplete();
              unsubError();
              reject(new Error(err));
            }
          });
        });
        
        // Hide progress dialog
        updateProgressVisible.set(false);
        
        // When complete, show success message and relaunch
        await message('Update has been downloaded and will be installed now. The application will restart.', {
          title: 'Update Ready'
        });
        
        await relaunch();
      } catch (error) {
        // Hide progress dialog
        updateProgressVisible.set(false);
        
        console.error('Update installation failed:', error);
        await message(`Failed to install update: ${error}`, {
          title: 'Update Error'
        });
      }
    } else {
      console.log('User declined the update');
    }
  } catch (error) {
    console.error('Update check failed:', error);
    await message(`Failed to check for updates: ${error}`, {
      title: 'Update Error'
    });
  }
}

// Called by the progress component when the update is complete
export function handleUpdateComplete() {
  console.log('Update completed successfully!');
  updateProgressVisible.set(false);
  
  // After update, Tauri will normally restart the app automatically
  // No need to call openMainWindow since we're not using a separate window anymore
}

// Called by the progress component when there's an error
export function handleUpdateError(error: string) {
  updateError.set(error);
}