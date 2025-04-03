#!/usr/bin/env python3
"""
Deploy MicroPython code to target embedded device.

This script automates the deployment of the Flight Simulator MicroPython client 
to ESP32, PyBoard, or other MicroPython-compatible devices.

Usage:
    python deploy_to_device.py [--port PORT] [--device {esp32,pyboard}]

Options:
    --port PORT     Serial port of the device (e.g., /dev/ttyUSB0, COM3)
    --device        Target device type (esp32 or pyboard)
"""

import argparse
import os
import shutil
import sys
import time
from pathlib import Path
from typing import List, Optional

try:
    import mpremote.main
    HAS_MPREMOTE = True
except ImportError:
    HAS_MPREMOTE = False
    print("Warning: mpremote not found. Install with 'pip install mpremote'")

try:
    from ampy.files import Files
    from ampy.pyboard import Pyboard
    HAS_AMPY = True
except ImportError:
    HAS_AMPY = False
    print("Warning: adafruit-ampy not found. Install with 'pip install adafruit-ampy'")


# Path to the root of the project
PROJECT_ROOT = Path(__file__).parent.parent.parent
MICROPYTHON_DIR = PROJECT_ROOT / "micropython"


def parse_args():
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(description="Deploy MicroPython code to target device")
    parser.add_argument(
        "--port",
        type=str,
        help="Serial port of the device (e.g., /dev/ttyUSB0, COM3)",
    )
    parser.add_argument(
        "--device",
        type=str,
        choices=["esp32", "pyboard"],
        default="esp32",
        help="Target device type",
    )
    parser.add_argument(
        "--use-ampy",
        action="store_true",
        help="Use ampy instead of mpremote",
    )
    return parser.parse_args()


def get_port() -> str:
    """Attempt to detect the serial port of the connected device."""
    import serial.tools.list_ports
    
    ports = list(serial.tools.list_ports.comports())
    
    if not ports:
        print("No serial ports found. Please specify a port with --port")
        sys.exit(1)
    
    # Try to identify ESP32 or PyBoard devices
    for port in ports:
        desc = port.description.lower()
        if "esp32" in desc or "cp210x" in desc or "ftdi" in desc or "pyboard" in desc:
            print(f"Found likely device at {port.device}")
            return port.device
    
    # If we couldn't identify, just use the first one and warn the user
    print(f"Using first available port: {ports[0].device}")
    print("If this is incorrect, specify the port with --port")
    return ports[0].device


def deploy_with_mpremote(port: str, files_to_deploy: List[Path]):
    """Deploy files using mpremote."""
    if not HAS_MPREMOTE:
        print("mpremote is not installed. Please install with 'pip install mpremote'")
        return False
    
    print(f"Deploying to device at {port} using mpremote...")
    
    try:
        # Create lib directory if it doesn't exist
        mpremote.main.main(["connect", port, "mkdir", "lib"])
    except Exception:
        # Directory might already exist, continue
        pass
    
    # Deploy each file
    for file_path in files_to_deploy:
        rel_path = file_path.relative_to(MICROPYTHON_DIR)
        print(f"Deploying {rel_path}...")
        
        # Create parent directories if needed
        if "/" in str(rel_path):
            parent_dir = str(rel_path).rsplit("/", 1)[0]
            try:
                mpremote.main.main(["connect", port, "mkdir", "-p", parent_dir])
            except Exception as e:
                print(f"Warning while creating directories: {e}")
        
        # Copy the file
        try:
            mpremote.main.main(["connect", port, "cp", str(file_path), ":" + str(rel_path)])
        except Exception as e:
            print(f"Error deploying {rel_path}: {e}")
            return False
    
    print("Deployment completed successfully!")
    return True


def deploy_with_ampy(port: str, files_to_deploy: List[Path]):
    """Deploy files using ampy."""
    if not HAS_AMPY:
        print("ampy is not installed. Please install with 'pip install adafruit-ampy'")
        return False
    
    print(f"Deploying to device at {port} using ampy...")
    
    try:
        board = Pyboard(port)
        files = Files(board)
        
        # Create lib directory if it doesn't exist
        try:
            files.mkdir("lib")
        except Exception:
            # Directory might already exist, continue
            pass
        
        # Deploy each file
        for file_path in files_to_deploy:
            rel_path = str(file_path.relative_to(MICROPYTHON_DIR))
            print(f"Deploying {rel_path}...")
            
            # Create parent directories if needed
            if "/" in rel_path:
                parent_dir = rel_path.rsplit("/", 1)[0]
                try:
                    dirs = parent_dir.split("/")
                    for i in range(len(dirs)):
                        try:
                            path = "/".join(dirs[:i+1])
                            files.mkdir(path)
                        except:
                            pass
                except Exception as e:
                    print(f"Warning while creating directories: {e}")
            
            # Copy the file
            with open(file_path, "rb") as f:
                content = f.read()
            try:
                files.put(rel_path, content)
            except Exception as e:
                print(f"Error deploying {rel_path}: {e}")
                return False
        
        print("Deployment completed successfully!")
        return True
        
    except Exception as e:
        print(f"Error connecting to the device: {e}")
        return False


def collect_files_to_deploy(device_type: str) -> List[Path]:
    """Collect all files that need to be deployed to the device."""
    files = []
    
    # Add the main client file
    client_file = MICROPYTHON_DIR / "client.py"
    files.append(client_file)
    
    # Add library files
    lib_dir = MICROPYTHON_DIR / "lib"
    for file_path in lib_dir.glob("**/*.py"):
        files.append(file_path)
    
    # Device-specific configuration file if it exists
    config_file = MICROPYTHON_DIR / f"config_{device_type}.py"
    if config_file.exists():
        files.append(config_file)
    
    return files


def main():
    """Main entry point."""
    args = parse_args()
    
    # Determine the port to use
    port = args.port
    if not port:
        try:
            port = get_port()
        except ImportError:
            print("pyserial not found. Please install with 'pip install pyserial'")
            print("Alternatively, specify a port with --port")
            sys.exit(1)
    
    # Collect files to deploy
    files_to_deploy = collect_files_to_deploy(args.device)
    
    if not files_to_deploy:
        print("No files found to deploy!")
        sys.exit(1)
    
    print(f"Found {len(files_to_deploy)} files to deploy...")
    
    # Deploy files using the preferred method
    if args.use_ampy or not HAS_MPREMOTE:
        success = deploy_with_ampy(port, files_to_deploy)
    else:
        success = deploy_with_mpremote(port, files_to_deploy)
    
    if not success:
        print("Deployment failed!")
        sys.exit(1)
    
    print("Deployment successful! Resetting device...")
    
    # Reset the device to run the new code
    try:
        if HAS_MPREMOTE and not args.use_ampy:
            mpremote.main.main(["connect", port, "reset"])
        else:
            # Try to reset using ampy or manually
            try:
                if HAS_AMPY:
                    board = Pyboard(port)
                    board.enter_raw_repl()
                    board.exec("import machine; machine.reset()")
            except:
                print("Could not automatically reset the device.")
                print("Please manually reset your device to run the new code.")
    except Exception as e:
        print(f"Error resetting the device: {e}")
        print("Please manually reset your device to run the new code.")


if __name__ == "__main__":
    main() 