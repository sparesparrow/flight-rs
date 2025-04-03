"""
Flight Simulator MicroPython Client

This client is designed to run on ESP32 or PyBoard devices and connects to
the Flight Simulator server via WebSockets. It uses hardware inputs (buttons/joystick)
to control an aircraft.

Dependencies:
- MicroPython with WebSocket support
- SSD1306 OLED display (128x64)
"""

import time
import json
import network
from machine import Pin, I2C, ADC
import uasyncio as asyncio

# Try to import the WebSocket client - this may vary by MicroPython implementation
try:
    import uwebsockets.client as websockets
except ImportError:
    import websockets.client as websockets

# Try to import the display library
try:
    import ssd1306
    HAS_DISPLAY = True
except ImportError:
    HAS_DISPLAY = False
    print("OLED display not available")

# Network configuration
WIFI_SSID = "YourWiFiSSID"  # Change to your WiFi network
WIFI_PASSWORD = "YourWiFiPassword"  # Change to your WiFi password
SERVER_URL = "ws://flight-server.local:8080/ws"  # Change to your server address

# Hardware pins (adjust for your specific board)
# For ESP32 example setup
BUTTON_UP = Pin(12, Pin.IN, Pin.PULL_UP)       # Up button
BUTTON_DOWN = Pin(14, Pin.IN, Pin.PULL_UP)     # Down button
BUTTON_THROTTLE_UP = Pin(27, Pin.IN, Pin.PULL_UP)  # Throttle up
BUTTON_THROTTLE_DOWN = Pin(26, Pin.IN, Pin.PULL_UP)  # Throttle down

# Joystick option (analog input)
try:
    JOYSTICK_Y = ADC(Pin(32))
    JOYSTICK_Y.atten(ADC.ATTN_11DB)  # Full range: 3.3V
    HAS_JOYSTICK = True
except:
    HAS_JOYSTICK = False
    print("Analog joystick not available")

# LED indicators
LED_CONNECTED = Pin(2, Pin.OUT)  # Blue LED for connection status
LED_THROTTLE = Pin(4, Pin.OUT)   # Green LED for throttle

# OLED Display setup (I2C - SDA=21, SCL=22 for ESP32 default)
if HAS_DISPLAY:
    i2c = I2C(0, scl=Pin(22), sda=Pin(21))
    oled = ssd1306.SSD1306_I2C(128, 64, i2c)

# Global state
client_id = None
aircraft_data = {
    "x": 0.0,
    "y": 0.0,
    "vx": 0.0,
    "vy": 0.0,
    "theta": 0.0,
    "throttle_level": 0.0
}

# Connect to WiFi network
async def connect_wifi():
    print(f"Connecting to WiFi: {WIFI_SSID}")
    
    wlan = network.WLAN(network.STA_IF)
    wlan.active(True)
    
    if not wlan.isconnected():
        wlan.connect(WIFI_SSID, WIFI_PASSWORD)
        
        # Wait for connection with timeout
        max_wait = 20
        while max_wait > 0:
            if wlan.isconnected():
                break
            max_wait -= 1
            print("Waiting for connection...")
            await asyncio.sleep(1)
        
        if wlan.isconnected():
            print("WiFi connected")
            print(f"IP address: {wlan.ifconfig()[0]}")
            return True
        else:
            print("WiFi connection failed")
            return False
    else:
        print("Already connected to WiFi")
        print(f"IP address: {wlan.ifconfig()[0]}")
        return True

# Update the OLED display with aircraft information
def update_display():
    if not HAS_DISPLAY:
        return
        
    oled.fill(0)
    oled.text("Flight Simulator", 0, 0)
    
    if client_id:
        oled.text(f"ID: {client_id[:8]}", 0, 10)
    else:
        oled.text("Not connected", 0, 10)
        
    oled.text(f"Alt: {aircraft_data['y']:.1f}m", 0, 20)
    oled.text(f"Spd: {(aircraft_data['vx']**2 + aircraft_data['vy']**2)**0.5:.1f}m/s", 0, 30)
    oled.text(f"Pitch: {aircraft_data['theta'] * 57.3:.1f}deg", 0, 40)
    oled.text(f"Throttle: {aircraft_data['throttle_level']*100:.0f}%", 0, 50)
    
    oled.show()

# Read input from hardware buttons/joystick
def read_input():
    input_state = {
        "pitch_up": False,
        "pitch_down": False,
        "throttle_up": False,
        "throttle_down": False
    }
    
    # Read buttons (buttons are pulled up, so not pressed = True)
    input_state["pitch_up"] = not BUTTON_UP.value()
    input_state["pitch_down"] = not BUTTON_DOWN.value()
    input_state["throttle_up"] = not BUTTON_THROTTLE_UP.value()
    input_state["throttle_down"] = not BUTTON_THROTTLE_DOWN.value()
    
    # Read joystick if available
    if HAS_JOYSTICK:
        # Normalize to -1 to 1 range (assuming middle position is around 2048)
        joy_y = (JOYSTICK_Y.read() - 2048) / 2048
        
        # Apply a dead zone to avoid drift
        if abs(joy_y) > 0.2:
            if joy_y < -0.2:
                input_state["pitch_up"] = True
            elif joy_y > 0.2:
                input_state["pitch_down"] = True
    
    # Update LED for throttle
    LED_THROTTLE.value(1 if input_state["throttle_up"] else 0)
    
    return input_state

# Main WebSocket client loop
async def ws_client():
    global client_id, aircraft_data
    
    while True:
        try:
            # Connect to WebSocket server
            print(f"Connecting to {SERVER_URL}")
            async with websockets.connect(SERVER_URL) as websocket:
                LED_CONNECTED.value(1)  # Turn on connection LED
                print("Connected to server")
                
                # Process messages
                while True:
                    # Check for and process messages from server
                    try:
                        message = await asyncio.wait_for(websocket.recv(), 0.1)
                        
                        # Process welcome message to get client ID
                        if message.startswith("Welcome!"):
                            client_id = message.split(":")[1].strip()
                            print(f"Received ID: {client_id}")
                        else:
                            # Process game state update
                            game_state = json.loads(message)
                            
                            # Find our aircraft in the list
                            if client_id and game_state.get("aircraft"):
                                for aircraft in game_state["aircraft"]:
                                    if aircraft["id"] == client_id:
                                        aircraft_data = aircraft
                                        break
                    except asyncio.TimeoutError:
                        # No message received in timeout period, continue
                        pass
                    
                    # Read inputs and send to server
                    input_state = read_input()
                    await websocket.send(json.dumps(input_state))
                    
                    # Update display
                    update_display()
                    
                    # Short delay
                    await asyncio.sleep(0.05)
                    
        except Exception as e:
            print(f"Connection error: {e}")
            LED_CONNECTED.value(0)  # Turn off connection LED
            client_id = None
            await asyncio.sleep(5)  # Wait before reconnecting

# Main function
async def main():
    if await connect_wifi():
        await ws_client()
    else:
        while True:
            LED_CONNECTED.toggle()
            await asyncio.sleep(0.5)

# Start the event loop
if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("Program terminated by user")
    finally:
        # Clean up
        if HAS_DISPLAY:
            oled.fill(0)
            oled.text("Disconnected", 0, 0)
            oled.show()
        LED_CONNECTED.value(0)
        LED_THROTTLE.value(0) 