#!/bin/bash
scriptpath=$(dirname $(realpath $0))
scriptname=$0

## check if unzip is installed
if ! command -v unzip &> /dev/null; then
  echo "unzip could not be found. Please install unzip."
  exit 1
fi

## for sharelatex download
sharelatex=false
domain=
email=
password=
project=
filename=

outputpath=

## Parse command line arguments
while getopts "D:e:p:P:f:o:h" opt; do
	case $opt in
			D)
					domain=$OPTARG
					;;
			e)
					email=$OPTARG
					;;
			p)
					password=$OPTARG
					;;
			P)
					project=$OPTARG
					;;
			f)
					filename=$OPTARG
					;;
			o)
					outputpath=$OPTARG
					;;
			h)
					echo "Usage: $scriptname -S -D <domain> -e <email> -p <password> -P <project> -f <filename>"
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

# remove filename from outputpath
zip="$outputpath/document.zip"
cookie="$outputpath/cookies.txt"

# Fetching csrf token from login page and saving it to csrf variable
echo "Fetching login page..."
curl -s -c $cookie "$domain/login" |
    sed -rn 's/^.*<input name="_csrf" type="hidden" value="([^"]+)".*$/\1/p' > $outputpath/csrf.txt
csrf=$(cat $outputpath/csrf.txt)

# Logging into sharelatex
echo "Logging into sharelatex..."
curl "$domain/login" -s -b $cookie -c $cookie -H "Referer: $domain/login" \
    -d "_csrf=$csrf" -d "email=$email" -d "password=$password"

# Download the project zip
echo -e "\nDownloading protocol..."
curl -b $cookie -c $cookie -o $zip $domain/project/$project/download/zip

# Unzip the project
echo -e "\nUnzipping protocol..."
unzip -q "$zip" "$filename" -d "$outputpath"

inputfile=$outputpath

