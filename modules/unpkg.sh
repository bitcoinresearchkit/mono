#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_debug() {
    if [[ "${DEBUG:-}" == "1" ]]; then
        echo -e "${YELLOW}[DEBUG]${NC} $1"
    fi
}

# Check if required tools are installed
check_dependencies() {
    local missing_deps=()

    if ! command -v curl &> /dev/null; then
        missing_deps+=("curl")
    fi

    if ! command -v grep &> /dev/null; then
        missing_deps+=("grep")
    fi

    if ! command -v sed &> /dev/null; then
        missing_deps+=("sed")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_error "Missing required dependencies: ${missing_deps[*]}"
        print_error "Please install them and try again."
        exit 1
    fi
}

# Function to URL encode a string
url_encode() {
    local string="${1}"
    local strlen=${#string}
    local encoded=""
    local pos c o

    for ((pos=0; pos<strlen; pos++)); do
        c=${string:$pos:1}
        case "$c" in
            [-_.~a-zA-Z0-9] ) o="${c}" ;;
            * )               printf -v o '%%%02x' "'$c"
        esac
        encoded+="${o}"
    done
    echo "${encoded}"
}

# Function to create directory if it doesn't exist
create_dir() {
    local dir="$1"
    if [ ! -d "$dir" ]; then
        mkdir -p "$dir"
        print_status "Created directory: $dir"
    fi
}

# Function to resolve "latest" version to actual version number
resolve_latest_version() {
    local package_name="$1"
    local version="$2"

    # If version is not "latest", return as-is
    if [[ "$version" != "latest" ]]; then
        echo "$version"
        return
    fi

    print_status "Resolving 'latest' version for $package_name..." >&2

    # URL encode the package name
    local encoded_package=$(url_encode "$package_name")
    local latest_url="https://app.unpkg.com/${encoded_package}@latest"

    # Use curl to follow redirects and get the final URL
    local final_url
    final_url=$(curl -L -s -o /dev/null -w '%{url_effective}' "$latest_url")

    if [[ -z "$final_url" ]]; then
        print_error "Failed to resolve latest version for $package_name" >&2
        return 1
    fi

    # Extract version from the final URL
    # Format: https://app.unpkg.com/@solidjs/signals@0.4.1
    local resolved_version
    # Use a different delimiter (#) to avoid issues with / in package names
    resolved_version=$(echo "$final_url" | sed -n "s#.*${package_name}@\([^\/]*\).*#\1#p")

    if [[ -z "$resolved_version" ]]; then
        print_error "Could not extract version from URL: $final_url" >&2
        return 1
    fi

    print_success "Resolved 'latest' to version: $resolved_version" >&2
    echo "$resolved_version"
}

# Function to download a file
download_file() {
    local file_url="$1"
    local local_path="$2"
    local dir=$(dirname "$local_path")

    create_dir "$dir"

    print_status "Downloading: $file_url"
    if curl -L -s -f "$file_url" -o "$local_path"; then
        print_success "Downloaded: $local_path"
        return 0
    else
        print_error "Failed to download: $file_url"
        return 1
    fi
}

# Function to parse HTML and extract file/folder links using a much simpler approach
parse_directory() {
    local html_content="$1"
    local current_path="$2"
    local package_name="$3"
    local version="$4"

    print_debug "Parsing directory for path: '$current_path'"

    # The HTML contains the original (unencoded) package name in URLs
    # So we search for the original package name, not the encoded version

    # Find all links that point to files/folders in this package
    # Look for the pattern: <a href="...">filename</a> or <a href="...">foldername/</a>
    local links=$(echo "$html_content" | grep -o '<a href="[^"]*"[^>]*>[^<]*</a>' | grep "${package_name}@${version}/files")

    print_debug "Found $(echo "$links" | wc -l) total links"

    if [[ "${DEBUG:-}" == "1" ]]; then
        print_debug "All found links:"
        echo "$links" | while read -r link; do
            [[ -n "$link" ]] && print_debug "  $link"
        done
    fi

    echo "$links" | while read -r link_line; do
        if [[ -z "$link_line" ]]; then
            continue
        fi

        # Extract the href URL
        local href=$(echo "$link_line" | sed -n 's/.*href="\([^"]*\)".*/\1/p')
        # Extract the link text (what's between <a> and </a>)
        local link_text=$(echo "$link_line" | sed -n 's/.*<a[^>]*>\([^<]*\)<\/a>.*/\1/p')

        print_debug "Processing link: href='$href' text='$link_text'"

        # Skip if we couldn't extract both parts
        if [[ -z "$href" ]] || [[ -z "$link_text" ]]; then
            continue
        fi

        # Skip parent directory links
        if [[ "$link_text" == "../" ]] || [[ "$link_text" == ".." ]]; then
            continue
        fi

        # Extract the file/folder path from the URL
        # URL format: https://app.unpkg.com/@solidjs/signals@0.4.1/files/path/to/item
        # Note: href contains the original (unencoded) package name
        local url_path="${href#*${package_name}@${version}/files}"
        url_path="${url_path#/}" # Remove leading slash

        print_debug "URL path extracted: '$url_path'"

        # Skip if this is not a direct child of current directory
        if [[ -n "$current_path" ]]; then
            # We're in a subdirectory, so the URL path should start with current_path
            local current_clean="${current_path#/}"
            if [[ "$url_path" != "${current_clean}"* ]]; then
                print_debug "Skipping - not in current path"
                continue
            fi
            # Get the relative path from current directory
            local relative_path="${url_path#${current_clean}}"
            relative_path="${relative_path#/}"
        else
            # We're in root, so relative_path is the same as url_path
            local relative_path="$url_path"
        fi

        print_debug "Relative path: '$relative_path'"

        # Skip if this contains subdirectories (we only want direct children)
        if [[ "$relative_path" == *"/"* ]]; then
            print_debug "Skipping - contains subdirectories"
            continue
        fi

        # Skip empty paths
        if [[ -z "$relative_path" ]]; then
            continue
        fi

        # Determine if it's a folder or file based on the link text
        if [[ "$link_text" == *"/" ]]; then
            # It's a folder
            local folder_name="${relative_path%/}"
            if [[ -n "$current_path" ]]; then
                echo "FOLDER:${current_path}/${folder_name}"
            else
                echo "FOLDER:/${folder_name}"
            fi
        else
            # It's a file
            if [[ -n "$current_path" ]]; then
                echo "FILE:${current_path}/${relative_path}"
            else
                echo "FILE:/${relative_path}"
            fi
        fi
    done
}

# Function to check if item was already processed
is_processed() {
    local item="$1"
    local processed_list="$2"

    [[ "$processed_list" == *"|${item}|"* ]]
}

# Function to add item to processed list
add_processed() {
    local item="$1"
    local processed_list="$2"

    echo "${processed_list}|${item}|"
}

# Function to process a directory (download files and recurse into subdirectories)
process_directory() {
    local package_name="$1"
    local version="$2"
    local dir_path="$3"
    local output_dir="$4"
    local processed_dirs="$5"

    # Encode the package name for URL
    local encoded_package=$(url_encode "$package_name")
    local app_url="https://app.unpkg.com/${encoded_package}@${version}/files${dir_path}"
    local download_base_url="https://unpkg.com/${encoded_package}@${version}"

    # Check if we've already processed this directory
    if is_processed "$dir_path" "$processed_dirs"; then
        print_warning "Already processed directory: $dir_path"
        return
    fi

    print_status "Processing directory: ${dir_path:-'(root)'}"
    print_status "Fetching: $app_url"

    # Download the directory listing HTML
    local html_content
    if ! html_content=$(curl -L -s -f "$app_url"); then
        print_error "Failed to fetch directory listing: $app_url"
        return
    fi

    print_status "Fetched HTML content (${#html_content} characters)"

    # Mark this directory as processed
    processed_dirs=$(add_processed "$dir_path" "$processed_dirs")

    # Parse the HTML to find files and folders
    local items
    items=$(parse_directory "$html_content" "$dir_path" "$package_name" "$version")

    print_debug "Parsed items:"
    print_debug "$items"

    # Collect unique files and folders
    local files=()
    local folders=()
    local seen_files=""
    local seen_folders=""

    while IFS= read -r item; do
        [[ -z "$item" ]] && continue

        if [[ "$item" == FILE:* ]]; then
            local file_path="${item#FILE:}"
            if ! is_processed "$file_path" "$seen_files"; then
                files+=("$file_path")
                seen_files=$(add_processed "$file_path" "$seen_files")
                print_debug "Added file: $file_path"
            fi
        elif [[ "$item" == FOLDER:* ]]; then
            local folder_path="${item#FOLDER:}"
            if ! is_processed "$folder_path" "$seen_folders"; then
                folders+=("$folder_path")
                seen_folders=$(add_processed "$folder_path" "$seen_folders")
                print_debug "Added folder: $folder_path"
            fi
        fi
    done <<< "$items"

    print_status "Found ${#files[@]} files and ${#folders[@]} folders"

    # Download all files in this directory
    for file_path in "${files[@]}"; do
        if [[ -n "$file_path" ]]; then
            local file_url="${download_base_url}${file_path}"
            local local_path="${output_dir}${file_path}"
            download_file "$file_url" "$local_path"
        fi
    done

    # Recursively process all folders
    for folder_path in "${folders[@]}"; do
        if [[ -n "$folder_path" ]]; then
            process_directory "$package_name" "$version" "$folder_path" "$output_dir" "$processed_dirs"
        fi
    done
}

# Main function
main() {
    # Check dependencies
    check_dependencies

    # Parse command line arguments
    if [ $# -lt 1 ] || [ $# -gt 3 ]; then
        echo "Usage: $0 <package-name> [version] [output-dir]"
        echo "Example: $0 \"@solidjs/signals\""
        echo "Example: $0 \"@solidjs/signals\" \"0.4.1\""
        echo "Example: $0 \"@solidjs/signals\" \"latest\""
        echo "Example: $0 \"@solidjs/signals\" \"latest\" \"./downloads\""
        echo "Example: $0 \"lodash\" \"4.17.21\" \"./downloads\""
        echo ""
        echo "Version defaults to 'latest' if not specified"
        echo "Set DEBUG=1 environment variable for verbose output"
        exit 1
    fi

    local package_name="$1"
    local version="${2:-latest}"

    # Resolve latest version if needed (do this once at the start)
    local resolved_version
    if ! resolved_version=$(resolve_latest_version "$package_name" "$version"); then
        exit 1
    fi

    # Use resolved version for output directory
    local output_dir="${3:-./$(echo "${package_name}" | sed 's/@//g' | sed 's/\//-/g')/${resolved_version}}"

    print_status "Starting download of package: $package_name@$version"
    if [[ "$version" == "latest" ]]; then
        print_status "Resolved to actual version: $resolved_version"
    fi
    print_status "Output directory: $output_dir"

    # Check if the directory already exists and has content
    if [[ -d "$output_dir" ]] && [[ -n "$(ls -A "$output_dir" 2>/dev/null)" ]]; then
        print_warning "Directory already exists and is not empty: $output_dir"
        print_warning "Package $package_name@$resolved_version appears to already be downloaded."
        print_warning "Remove the directory or choose a different output location to proceed."
        return
    fi

    # Create the base output directory
    create_dir "$output_dir"

    # Start processing from the root directory using the resolved version
    process_directory "$package_name" "$resolved_version" "" "$output_dir" ""

    print_success "Package download completed!"
    print_status "Files downloaded to: $output_dir"
}

# Run the main function with all arguments
# main "$@"

main "quickmatch-js"
main "lean-qr"
main "lightweight-charts"
main "world-atlas" "2.0.2"
