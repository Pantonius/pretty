#!/bin/bash

# Parse arguments
scriptpath=$(dirname $(realpath $0))
scriptname=$0

# get filepath
filepath=$1
outputpath=$filepath

## Parse command line arguments
while getopts "o:h" opt; do
	case $opt in
			o)
				outputpath=$OPTARG
				;;
			h)
				echo "Usage: $scriptname -o <outputpath>"
				exit 0
				;;
			\?)
				echo "Invalid option: $OPTARG" 1>&2
				exit 1
				;;
			:)
				echo "Option -$OPTARG requires an argument." 1>&2
				exit 1
				;;
	esac
done


# Parse frontmatter
parse_frontmatter() {
    # extract key value pairs
    while read -r line; do
        key=$(echo $line | cut -d: -f1 | tr -d '[:space:]')
	value=$(echo $line | cut -d: -f2- | sed -r 's/^\s*"?([^"]*)"?$/\1/')
        case $key in
            title) name=$value ;;
            font) font=$value ;;
            logo) logo=$value ;;
            tocTitle) tocTitle=$value ;;
            tocSubtitle) tocSubtitle=$value ;;
        esac
    done <<< $(echo "$1" | sed -n '2,$p')
}

frontmatter=$(sed -n '/^---$/,/^---$/p' $filepath)
if [ -n "$frontmatter" ]; then
    echo "Parsing frontmatter..."

    # parse
    parse_frontmatter "$frontmatter"

    # remove frontmatter from the file
    echo "Saving frontmatter to $outputpath..."
    sed -i '/^---$/,/^---$/d' $outputpath
fi
