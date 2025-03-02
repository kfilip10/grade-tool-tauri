import { getCurrentWindow } from '@tauri-apps/api/window';
import { stopShinyApp } from './shiny';
import { shinyStatus, shinyUrl } from './shinyListener';
import { get } from 'svelte/store';
import { initStatus } from './initialization';

// Flag to track if monitoring is active
let isMonitoring = false;
let intervalId: number | null = null;

/**
 * Opens the Shiny application in the default browser
 */
export async function openBrowserWindow(shinyUrl: string): Promise<void> {
  console.log("Opening Shiny URL in browser:", shinyUrl);
  
  try {
    // Stop any existing monitoring
    stopBrowserMonitoring();
    
    // Open the URL in the default browser
    await open(shinyUrl);
    
    // Hide the Tauri window after opening the browser
    await hideCurrentWindow();
    
    // Set up a periodic check for browser closure
    setupBrowserMonitoring(shinyUrl);
    
  } catch (e) {
    console.error("Error opening browser:", e);
  }
}

/**
 * Monitor if the user has closed the browser tab
 * Uses a ping approach to check if the Shiny app is still accessible
 */
function setupBrowserMonitoring(shinyUrl: string): void {
  if (isMonitoring) {
    stopBrowserMonitoring();
  }
  
  let pingCount = 0;
  const maxMissedPings = 3; // Number of failed pings before assuming browser closed
  let missedPings = 0;
  isMonitoring = true;
  
  console.log("Starting browser monitoring for Shiny URL:", shinyUrl);
  
  // Set up ping interval (every 3 seconds)
  intervalId = window.setInterval(async () => {
    try {
      pingCount++;
      console.log(`Ping #${pingCount} to Shiny app`);
      
      // Try to fetch the Shiny URL
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 2000); // 2 second timeout
      
      try {
        // Use GET request with cache busting to avoid cached responses
        const response = await fetch(`${shinyUrl}?_=${Date.now()}`, { 
          method: 'HEAD',
          mode: 'no-cors',
          signal: controller.signal
        });
        
        clearTimeout(timeoutId);
        
        // If we get here, the connection succeeded
        missedPings = 0;
        console.log("Shiny app is still running");
      } catch (fetchError) {
        clearTimeout(timeoutId);
        throw fetchError; // Re-throw to be caught by outer catch
      }
      
    } catch (e) {
      // Failed to connect, increment missed pings
      missedPings++;
      console.log(`Failed to connect to Shiny app (${missedPings}/${maxMissedPings}): ${e}`);
      
      // If we've missed too many pings, assume the browser is closed
      if (missedPings >= maxMissedPings) {
        console.log("Browser appears to be closed, stopping Shiny app");
        stopBrowserMonitoring();
        
        // Only stop if Shiny is still running according to our state
        if (get(shinyStatus) === 'running') {
        // Update UI first
        initStatus.set('starting');
        
        // Then stop the app
        try {
          await stopShinyApp(); // Make sure to await this
          console.log("Shiny app stopped, closing Tauri window");
          
          // Close the Tauri window after a delay to ensure cleanup
          setTimeout(() => {
            try {
              console.log("Closing Tauri window now");
              const window = getCurrentWindow();
              window.close();
            } catch (err) {
              console.error("Error closing Tauri window:", err);
            }
          }, 1000); // Give a slightly longer delay (1 second)
        } catch (err) {
          console.error("Error stopping Shiny app:", err);
          // Still try to close the window even if stopping Shiny fails
          setTimeout(() => getCurrentWindow().close(), 1000);
        }
        }
      }
    }
  }, 2000);
}

/**
 * Stop the browser monitoring
 */
export function stopBrowserMonitoring(): void {
  if (intervalId !== null) {
    console.log("Stopping browser monitoring");
    window.clearInterval(intervalId);
    intervalId = null;
    isMonitoring = false;
  }
}

/**
 * Close the current window
 */
export async function closeCurrentWindow(): Promise<void> {
  try {
    // Make sure to stop monitoring
    stopBrowserMonitoring();
    
    const currentWindow = getCurrentWindow();
    await currentWindow.close();
  } catch (e) {
    console.error('Error closing current window:', e);
  }
}
/**
 * Shows the current window and optionally brings it to front
 */
export async function showCurrentWindow(): Promise<void> {
  try {
    const currentWindow = getCurrentWindow();
    await currentWindow.show();
    await currentWindow.setFocus();
  } catch (e) {
    console.error('Error showing window:', e);
  }
}

/**
 * Hides the current window
 */
export async function hideCurrentWindow(): Promise<void> {
  try {
    const currentWindow = getCurrentWindow();
    await currentWindow.hide();
  } catch (e) {
    console.error('Error hiding window:', e);
  }
}