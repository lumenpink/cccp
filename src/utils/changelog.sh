#!/bin/sh

# Enable error handling
set -eu

# Get the git root directory
GIT_ROOT=$(git rev-parse --show-toplevel)
if [ $? -ne 0 ]; then
    echo "Error: Not a git repository" >&2
    exit 1
fi

# Source the configuration file
. "$GIT_ROOT/src/config/config.sh"

# Function to format commit message with scope
format_commit_message() {
    local msg="$1"
    # Extract scope and message, then format with scope in parentheses
    if echo "$msg" | grep -q "("; then
        # Has scope
        local scope=$(echo "$msg" | sed -E 's/^[a-z]+\(([^)]+)\):.*/\1/')
        local message=$(echo "$msg" | sed -E 's/^[a-z]+\([^)]+\): (.*)/\1/')
        echo "  - ($scope) $message"
    else
        # No scope
        echo "$msg" | sed -E 's/^[a-z]+: /  - /'
    fi
}

# Function to generate a changelog based on conventional commits
generate_changelog() {
    local changelog_file="CHANGELOG.md"

    # Get all commits since the last tag
    local last_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
    local commits
    if [ -n "$last_tag" ]; then
        commits=$(git log "$last_tag"..HEAD --pretty=format:"%s")
    else
        commits=$(git log --pretty=format:"%s")
    fi

    # Extract commits by type
    local feat_commits=$(echo "$commits" | grep "^feat" | while read -r commit; do format_commit_message "$commit"; done || echo "")
    local fix_commits=$(echo "$commits" | grep "^fix" | while read -r commit; do format_commit_message "$commit"; done || echo "")
    local perf_commits=$(echo "$commits" | grep "^perf" | while read -r commit; do format_commit_message "$commit"; done || echo "")

    # Initialize changelog file with header and unreleased section
    {
        echo "# Changelog"
        echo
        echo "## [Unreleased]"
        echo
        echo "### Features"
        
        # Add feature commits
        if [ -n "$feat_commits" ]; then
            echo "$feat_commits"
        fi
        
        echo
        echo "### Bug Fixes"
        
        # Add bug fix commits
        if [ -n "$fix_commits" ]; then
            echo "$fix_commits"
        fi
        
        echo
        echo "### Performance Improvements"
        
        # Add performance improvement commits
        if [ -n "$perf_commits" ]; then
            echo "$perf_commits"
        fi
        
        echo
        echo "## Previous Releases"
        echo
    } > "$changelog_file"

    # Get all tags sorted by version
    local tags=$(git tag -l --sort=-v:refname)
    if [ -n "$tags" ]; then
        local prev_tag=""
        for tag in $tags; do
            if [ -n "$prev_tag" ]; then
                # Get commits between tags
                local tag_commits=$(git log "$tag..$prev_tag" --pretty=format:"%s")
                
                # Extract commits by type for this tag range
                local tag_feat_commits=$(echo "$tag_commits" | grep "^feat" | while read -r commit; do format_commit_message "$commit"; done || echo "")
                local tag_fix_commits=$(echo "$tag_commits" | grep "^fix" | while read -r commit; do format_commit_message "$commit"; done || echo "")
                local tag_perf_commits=$(echo "$tag_commits" | grep "^perf" | while read -r commit; do format_commit_message "$commit"; done || echo "")
                
                # Add tag section
                {
                    echo "### [$tag]"
                    echo
                    echo "### Features"
                    
                    # Add feature commits
                    if [ -n "$tag_feat_commits" ]; then
                        echo "$tag_feat_commits"
                    fi
                    
                    echo
                    echo "### Bug Fixes"
                    
                    # Add bug fix commits
                    if [ -n "$tag_fix_commits" ]; then
                        echo "$tag_fix_commits"
                    fi
                    
                    echo
                    echo "### Performance Improvements"
                    
                    # Add performance improvement commits
                    if [ -n "$tag_perf_commits" ]; then
                        echo "$tag_perf_commits"
                    fi
                    
                    echo
                } >> "$changelog_file"
            fi
            prev_tag="$tag"
        done

        # Handle the last tag
        if [ -n "$prev_tag" ]; then
            # Get commits before the first tag
            local first_commits=$(git log "$prev_tag" --pretty=format:"%s")
            
            # Extract commits by type for the first tag
            local first_feat_commits=$(echo "$first_commits" | grep "^feat" | while read -r commit; do format_commit_message "$commit"; done || echo "")
            local first_fix_commits=$(echo "$first_commits" | grep "^fix" | while read -r commit; do format_commit_message "$commit"; done || echo "")
            local first_perf_commits=$(echo "$first_commits" | grep "^perf" | while read -r commit; do format_commit_message "$commit"; done || echo "")
            
            # Add the first tag section
            {
                echo "### [$prev_tag]"
                echo
                echo "### Features"
                
                # Add feature commits
                if [ -n "$first_feat_commits" ]; then
                    echo "$first_feat_commits"
                fi
                
                echo
                echo "### Bug Fixes"
                
                # Add bug fix commits
                if [ -n "$first_fix_commits" ]; then
                    echo "$first_fix_commits"
                fi
                
                echo
                echo "### Performance Improvements"
                
                # Add performance improvement commits
                if [ -n "$first_perf_commits" ]; then
                    echo "$first_perf_commits"
                fi
            } >> "$changelog_file"
        fi
    fi
} 