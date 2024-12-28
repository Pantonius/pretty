#!/bin/bash

scriptpath=$(dirname $(realpath $0))
scriptname=$0

# ensure dependencies are installed
## pandoc
if ! command -v pandoc &> /dev/null; then
    echo "pandoc could not be found. Please install pandoc."
    exit 1
fi

## xelatex
if ! command -v xelatex &> /dev/null; then
    echo "xelatex could not be found. Please install xelatex."
    exit 1
fi

## git
if ! command -v git &> /dev/null; then
    echo "git could not be found. Please install git."
    exit 1
fi

## Ubuntu Font
if ! fc-list | grep -q "Ubuntu"; then
    echo "Ubuntu font could not be found. Please install the Ubuntu font."
    exit 1
fi

# OPTS_SPEC is a string that describes the command-line options for the script, based on the requirements of git rev-parse --parseopt.
OPTS_SPEC="\
${scriptname} [<options>] [--] [<inputfile>]

Pretty Proto - Compile a protocol from a markdown file

inputfile   The markdown file to compile (if the download flag isn't set). If not provided, input is taken from stdin instead.
--
h,help        show this help

S,sharelatex  download the protocol from sharelatex
H,hedgedoc    download the protocol from hedgedoc
k,keep        keep the downloaded markdown protocol
e,email=      the email to use for downloading the protocol from sharelatex
p,password=   the password to use for downloading the protocol from sharelatex
D,domain=     the domain of the sharelatex or hedgedoc instance
P,project=    the project id of the protocol on sharelatex
f,filename=   the filename of the protocol on sharelatex
I,id=         the id of the protocol on hedgedoc
c,chair=      add the signature of the chair to the
t,transcript= add the signature of the transcript writer to the protocol
s,show        show the compiled pdf"

# Create tmpdir
tmpdir=$(mktemp -d)
chmod 700 $tmpdir

# Set default values
font="Ubuntu"                           # The font to use for the pdf
logo=""         			# The logo to use for the pdf
tocTitle="Table of Contents"            # The title of the table of contents
tocSubtitle=""                          # The subtitle of the table of contents
show=false                              # Show the compiled pdf

## DOWNLOAD
download=false
keep=false
domain=

## for sharelatex download
sharelatex=false
sl_domain=
email=
password=
project=
filename=

## for hedgedoc download
hedgedoc=false
hd_domain=
id=

# pretty.conf is a file with a pretty configuration
if [ -f pretty.conf ]; then
    # read the configuration file
    echo "Reading configuration file pretty.conf..."

    # source the configuration file
    source ./pretty.conf
fi


# Function to parse the arguments via git rev-parse --parseopt
# Based on https://www.lucas-viana.com/posts/bash-argparse/#a-fully-functional-copypaste-example
parse_args() {
    set_args="$(echo "$OPTS_SPEC" | git rev-parse --parseopt -- "$@" || echo exit $?)"

    eval "$set_args"
    
    while (( $# > 2 )); do
        opt=$1
        shift
        case "$opt" in
            -S|--sharelatex) download=true; sharelatex=true ;;
            -H|--hedgedoc) download=true; hedgedoc=true ;;
            -k|--keep) keep=true ;;
            -e|--email) email=$1; shift ;;
            -p|--password) password=$1; shift ;;
            -D|--domain) domain=$1; shift ;;
            -P|--project) project=$1; shift ;;
            -f|--filename) filename=$1; shift ;;
            -I|--id) id=$1; shift ;;
            -s|--show) show=true ;;
        esac
    done

    inputfile="$2"
}

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
	    sigline) sigline=$value ;;
        esac
    done <<< $(echo "$1" | sed -n '2,$p')
}

# Parse the arguments
parse_args "$@"

# If the download and sharelatex flags are set, download the protocol from sharelatex
if [ "$download" = true ] && [ "$sharelatex" = true ]; then
    # set the domain if not set
    if [ -z "$domain" ]; then
        domain=$sl_domain
    fi

    # Download the protocol from sharelatex
    sh $scriptpath/downloaders/sharelatex.sh -D $domain -e $email -p $password -P $project -f $filename -o $tmpdir/document.md

    inputfile=$tmpdir/$filename
fi

# If the download and hedgedoc flags are set, download the protocol from hedgedoc
if [ "$download" = true ] && [ "$hedgedoc" = true ]; then
    # set the domain if not set
    if [ -z "$domain" ]; then
        domain=$hd_domain
    fi

    # Download the protocol from hedgedoc
    sh $scriptpath/downloaders/hedgedoc.sh -D $domain -I $id -o $tmpdir/document.md
    inputfile=$tmpdir/document.md
fi

# If inputfile is not set and no stdin is provided, show usage
if [ -z "$inputfile" ] && [ -t 0 ]; then
    echo "No input file provided and no stdin provided."
    exit 1
fi

# Create a working copy of the input file
tmpfile="$tmpdir/$inputfile"
if [ -n "$inputfile" ]; then
    # if inputfile is already within the tmpdir, leave it
    if [ "${inputfile:0:${#tmpdir}}" = "$tmpdir" ]; then
        tmpfile=$inputfile
    else
        # if inputfile is not within the tmpdir, copy it
        echo "Creating working copy of $inputfile..."
        cp $inputfile $tmpfile
    fi
else
    # take from stdin
    echo "Taking input from stdin..."
    tmpfile="$tmpdir/stdin"
    cat > $tmpfile
    inputfile="stdin"
fi

# Parse frontmatter
frontmatter=$(sed -n '/^---$/,/^---$/p' $tmpfile)
if [ -n "$frontmatter" ]; then
    echo "Parsing frontmatter..."

    # parse
    parse_frontmatter "$frontmatter"

    # remove frontmatter from the file
    sed -i '/^---$/,/^---$/d' $tmpfile
fi

# If the name is not set (by the frontmatter), try to find it in the text
if [ -z "$name" ]; then
    # Try to find the name of the protocol
    name=$(grep -E ".?Az\." $tmpfile | sed -E 's/.*Az\.\s*(.*Protokoll).*/\1/')
    if [ -z "$name" ]; then
        echo "Could not figure out the name of the protocol."
        name="document"
    fi
fi

# If the download and keep flags are set, keep the markdown file
if [ "$download" = true ] && [ "$keep" = true ]; then
    echo "Keeping the markdown file..."
    cp $tmpfile $name.md
fi

# Set the output file names
pdf="$name.pdf"
latex="$name.tex"

# compile to pdf
echo Compiling to $pdf

# pandoc "$tmpfile" \
#     -f markdown \
#     --template="$scriptpath/tex/template.tex" \
#     --include-in-header="$scriptpath/tex/style.tex" \
#     -V logo:"$logo" \
#     -V header:"$(echo $name | sed -E 's/[_]/\\_/g')" \
#     -V mainfont="$font" \
#     -V colorlinks:true \
#     -V linkcolor:darkbluk \
#     -V urlcolor:darkbluk \
#     -V toccolor:black \
#     -V toc-title:"$tocTitle" \
#     -V toc-subtitle:"$tocSubtitle" \
#     -V toc-depth:1 \
#     -t latex \
#     -o "test.tex"

pandoc "$tmpfile" \
    -f markdown \
    --template="$scriptpath/tex/template.tex" \
    --include-in-header="$scriptpath/tex/style.tex" \
    -V logo:"$logo" \
    -V header:"$(echo $name | sed -E 's/[_]/\\_/g')" \
    -V mainfont="$font" \
    -V colorlinks:true \
    -V linkcolor:darkbluk \
    -V urlcolor:darkbluk \
    -V toccolor:black \
    -V toc-title:"$tocTitle" \
    -V toc-subtitle:"$tocSubtitle" \
    -V toc-depth:1 \
    -V lang:de \
    -V csquotes:true \
    -t pdf \
    --pdf-engine=xelatex \
    -o "$pdf"

# show the pdf if the -s flag is set
if [ "$show" = true ]; then
    echo "Opening $pdf..."
    xdg-open "$pdf"
fi

# cleanup
rm -r $tmpdir
