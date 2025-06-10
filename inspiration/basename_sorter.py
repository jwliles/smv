#!/usr/bin/env python3

import os
import shutil
from collections import defaultdict

def create_and_move_files(directory='.'):
    # Step 1: Scan all file and folder names
    file_map = defaultdict(list)
    folders = set()

    for item in os.listdir(directory):
        item_path = os.path.join(directory, item)

        # Skip hidden files and folders
        if item.startswith('.'):
            continue

        # Track folders separately
        if os.path.isdir(item_path):
            folders.add(item)
        else:
            # Track files by base name
            base_name = os.path.splitext(item)[0]
            file_map[base_name].append(item)

    # Step 2: Create folders for each unique base name if not already present
    for base_name in file_map.keys():
        subdirectory = os.path.join(directory, base_name)
        if not os.path.exists(subdirectory):
            os.makedirs(subdirectory, exist_ok=True)
            print(f'Created folder: {subdirectory}')

    # Step 3: Move files with base names into their corresponding folders
    for base_name, files in file_map.items():
        target_folder = os.path.join(directory, base_name)

        for file_name in files:
            source_path = os.path.join(directory, file_name)
            destination_path = os.path.join(target_folder, file_name)

            # Move file to its corresponding folder
            shutil.move(source_path, destination_path)
            print(f'Moved {source_path} to {destination_path}')

if __name__ == '__main__':
    create_and_move_files()
