#!/bin/bash
scriptpath=$(dirname $(realpath $0))
scriptname=$0

## for hedgedoc download
domain=
id=
outputpath=

## Parse command line arguments
while getopts "D:I:o:h" opt; do
	case $opt in
			D)
					domain=$OPTARG
					;;
			I)
					id=$OPTARG
					;;
			o)
					outputpath=$OPTARG
					;;
			h)
					echo "Usage: $scriptname -D <domain> -I <id> -o <outputpath>"
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

url="$domain/$id/download"

# Download the protocol from hedgedoc
echo -e "\nDownloading protocol from $url..."
curl -s -o $outputpath $url
