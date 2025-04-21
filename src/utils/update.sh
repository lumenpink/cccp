#!/bin/sh

# Source the configuration
. "$(dirname "$(dirname "$0")")/config/config.sh"

# Function to update the script
update_script() {
    echo "Updating cccp.sh from $UPDATE_URL..."
    
    # Download the new script
    if ! wget -q "$UPDATE_URL" -O "$GIT_ROOT/cccp.sh.new"; then
        echo "Error: Failed to download the new script"
        return 1
    fi
    
    # Make the new script executable
    chmod +x "$GIT_ROOT/cccp.sh.new"
    
    # Backup the current script
    if [ -f "$GIT_ROOT/cccp.sh" ]; then
        mv "$GIT_ROOT/cccp.sh" "$GIT_ROOT/cccp.sh.bak"
    fi
    
    # Replace the current script with the new one
    mv "$GIT_ROOT/cccp.sh.new" "$GIT_ROOT/cccp.sh"
    
    echo "Successfully updated cccp.sh"
    echo "A backup of your previous version was saved as cccp.sh.bak"
    
    return 0
}

# Execute the update
update_script 