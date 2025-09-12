#!/bin/bash

# Check if a commit message was provided
if [ $# -eq 0 ]; then
    echo "Error: Please provide a commit message"
    echo "Usage: ./deploy.sh \"Your commit message\""
    exit 1
fi

# Use the first argument as the commit message
commit_message="$1"

git add .
git commit -m "$commit_message"
git push 

# echo "Deployed with message: $commit_message"