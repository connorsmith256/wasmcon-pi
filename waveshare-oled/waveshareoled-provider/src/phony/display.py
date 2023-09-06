#!/usr/bin/python

import RPi.GPIO as GPIO
import time
from smbus import SMBus
import spidev

import ctypes

from PIL import Image,ImageDraw,ImageFont

import numpy as np
import sys
from enum import Enum

LCD_WIDTH   = 128 #LCD width
LCD_HEIGHT  = 64  #LCD height

# Pin definition
RST_PIN         = 25
DC_PIN          = 24
CS_PIN          = 8
BL_PIN          = 18
JS_UP_PIN = 6  #Joystick Up
JS_DOWN_PIN = 19 #Joystick Down
JS_LEFT_PIN = 5  #Joystick Left
JS_RIGHT_PIN = 26 #Joystick Right
JS_PRESSED_PIN = 13 #Joystick Pressed
BTN1_PIN = 21
BTN2_PIN = 20
BTN3_PIN = 16

Device_SPI = 1
Device_I2C = 0

if(Device_SPI == 1):
    Device = Device_SPI
    spi = spidev.SpiDev(0, 0)
else :
    Device = Device_I2C
    address         = 0x3C
    bus = SMBus(1)

def digital_write(pin, value):
    GPIO.output(pin, value)

def digital_read(pin):
    return GPIO.input(BUSY_PIN)

def delay_ms(delaytime):
    time.sleep(delaytime / 1000.0)

def spi_writebyte(data):
    # SPI.writebytes(data)
    spi.writebytes([data[0]])

def i2c_writebyte(reg, value):
    bus.write_byte_data(address, reg, value)

# Event handling functions

class Event(Enum):
    BTN1_PRESS = "button1"
    BTN2_PRESS = "button2"
    BTN3_PRESS = "button3"
    JS_UP = "joystick_up"
    JS_DOWN = "joystick_down"
    JS_LEFT = "joystick_left"
    JS_RIGHT = "joystick_right"
    JS_PRESSED = "joystick_pressed"

def b1_press(_channel):
    print(Event.BTN1_PRESS.value)

def b2_press(_channel):
    print(Event.BTN2_PRESS.value)

def b3_press(_channel):
    print(Event.BTN3_PRESS.value)

def js_up(_channel):
    print(Event.JS_UP.value)

def js_down(_channel):
    print(Event.JS_DOWN.value)

def js_left(_channel):
    print(Event.JS_LEFT.value)

def js_right(_channel):
    print(Event.JS_RIGHT.value)

def js_pressed(_channel):
    print(Event.JS_PRESSED.value)
    
    # time.sleep(0.01)
def module_init():
    # print("module_init")

    GPIO.setmode(GPIO.BCM)
    GPIO.setwarnings(False)
    GPIO.setup(RST_PIN, GPIO.OUT)
    GPIO.setup(DC_PIN, GPIO.OUT)
    GPIO.setup(CS_PIN, GPIO.OUT)
    GPIO.setup(BL_PIN, GPIO.OUT)
    GPIO.setup(JS_UP_PIN, GPIO.IN, pull_up_down=GPIO.PUD_UP) # Input with pull-up
    GPIO.setup(JS_DOWN_PIN, GPIO.IN, pull_up_down=GPIO.PUD_UP) # Input with pull-up
    GPIO.setup(JS_LEFT_PIN, GPIO.IN, pull_up_down=GPIO.PUD_UP) # Input with pull-up
    GPIO.setup(JS_RIGHT_PIN, GPIO.IN, pull_up_down=GPIO.PUD_UP) # Input with pull-up
    GPIO.setup(JS_PRESSED_PIN, GPIO.IN, pull_up_down=GPIO.PUD_UP) # Input with pull-up
    GPIO.setup(BTN1_PIN, GPIO.IN, pull_up_down=GPIO.PUD_UP) # Input with pull-up
    GPIO.setup(BTN2_PIN, GPIO.IN, pull_up_down=GPIO.PUD_UP) # Input with pull-up
    GPIO.setup(BTN3_PIN, GPIO.IN, pull_up_down=GPIO.PUD_UP) # Input with pull-up

    GPIO.add_event_detect(BTN1_PIN, GPIO.RISING, callback=b1_press, bouncetime=200)
    GPIO.add_event_detect(BTN2_PIN, GPIO.RISING, callback=b2_press, bouncetime=200)
    GPIO.add_event_detect(BTN3_PIN, GPIO.RISING, callback=b3_press, bouncetime=200)
    GPIO.add_event_detect(JS_LEFT_PIN, GPIO.RISING, callback=js_left, bouncetime=200)
    GPIO.add_event_detect(JS_RIGHT_PIN, GPIO.RISING, callback=js_right, bouncetime=200)
    GPIO.add_event_detect(JS_UP_PIN, GPIO.RISING, callback=js_up, bouncetime=200)
    GPIO.add_event_detect(JS_DOWN_PIN, GPIO.RISING, callback=js_down, bouncetime=200)
    GPIO.add_event_detect(JS_PRESSED_PIN, GPIO.RISING, callback=js_pressed, bouncetime=200)
    
    # SPI.max_speed_hz = 2000000
    # SPI.mode = 0b00
    # i2c_writebyte(0xff,0xff)
    if(Device == Device_SPI):
        # spi.SYSFS_software_spi_begin()
        # spi.SYSFS_software_spi_setDataMode(0);
        # spi.SYSFS_software_spi_setClockDivider(1);
        spi.max_speed_hz = 10000000
        spi.mode = 0b00
    
    GPIO.output(CS_PIN, 0)
    GPIO.output(BL_PIN, 1)
    GPIO.output(DC_PIN, 0)
    return 0

def module_exit():
    if(Device == Device_SPI):
        spi.SYSFS_software_spi_end()
    else :
        bus.close()
    GPIO.output(RST_PIN, 0)
    GPIO.output(DC_PIN, 0)

class SH1106(object):
    def __init__(self):
        self.width = LCD_WIDTH
        self.height = LCD_HEIGHT
        #Initialize DC RST pin
        self._dc = DC_PIN
        self._rst = RST_PIN
        self._bl = BL_PIN
        self.Device = Device

    """    Write register address and data     """
    def command(self, cmd):
        if(self.Device == Device_SPI):
            GPIO.output(self._dc, GPIO.LOW)
            spi_writebyte([cmd])
        else:
            i2c_writebyte(0x00, cmd)

    def Init(self):
        if (module_init() != 0):
            return -1
        """Initialize dispaly"""    
        self.reset()
        self.command(0xAE);#--turn off oled panel
        self.command(0x02);#---set low column address
        self.command(0x10);#---set high column address
        self.command(0x40);#--set start line address  Set Mapping RAM Display Start Line (0x00~0x3F)
        self.command(0x81);#--set contrast control register
        self.command(0xA0);#--Set SEG/Column Mapping     
        self.command(0xC0);#Set COM/Row Scan Direction   
        self.command(0xA6);#--set normal display
        self.command(0xA8);#--set multiplex ratio(1 to 64)
        self.command(0x3F);#--1/64 duty
        self.command(0xD3);#-set display offset    Shift Mapping RAM Counter (0x00~0x3F)
        self.command(0x00);#-not offset
        self.command(0xd5);#--set display clock divide ratio/oscillator frequency
        self.command(0x80);#--set divide ratio, Set Clock as 100 Frames/Sec
        self.command(0xD9);#--set pre-charge period
        self.command(0xF1);#Set Pre-Charge as 15 Clocks & Discharge as 1 Clock
        self.command(0xDA);#--set com pins hardware configuration
        self.command(0x12);
        self.command(0xDB);#--set vcomh
        self.command(0x40);#Set VCOM Deselect Level
        self.command(0x20);#-Set Page Addressing Mode (0x00/0x01/0x02)
        self.command(0x02);#
        self.command(0xA4);# Disable Entire Display On (0xa4/0xa5)
        self.command(0xA6);# Disable Inverse Display On (0xa6/a7) 
        time.sleep(0.1)
        self.command(0xAF);#--turn on oled panel
        
   
    def reset(self):
        """Reset the display"""
        GPIO.output(self._rst,GPIO.HIGH)
        time.sleep(0.1)
        GPIO.output(self._rst,GPIO.LOW)
        time.sleep(0.1)
        GPIO.output(self._rst,GPIO.HIGH)
        time.sleep(0.1)
    
    def getbuffer(self, image):
        # print "bufsiz = ",(self.width/8) * self.height
        buf = [0xFF] * ((self.width//8) * self.height)
        image_monocolor = image.convert('1')
        imwidth, imheight = image_monocolor.size
        pixels = image_monocolor.load()
        # print "imwidth = %d, imheight = %d",imwidth,imheight
        if(imwidth == self.width and imheight == self.height):
            # print ("Vertical")
            for y in range(imheight):
                for x in range(imwidth):
                    # Set the bits for the column of pixels at the current position.
                    if pixels[x, y] == 0:
                        buf[x + (y // 8) * self.width] &= ~(1 << (y % 8))
                        # print x,y,x + (y * self.width)/8,buf[(x + y * self.width) / 8]
                        
        elif(imwidth == self.height and imheight == self.width):
            # print ("Vertical")
            for y in range(imheight):
                for x in range(imwidth):
                    newx = y
                    newy = self.height - x - 1
                    if pixels[x, y] == 0:
                        buf[(newx + (newy // 8 )*self.width) ] &= ~(1 << (y % 8))
        return buf
            
    def ShowImage(self, pBuf):
        for page in range(0,8):
            # set page address #
            self.command(0xB0 + page);
            # set low column address #
            self.command(0x02); 
            # set high column address #
            self.command(0x10); 
            # write data #
            # time.sleep(0.01)
            if(self.Device == Device_SPI):
                GPIO.output(self._dc, GPIO.HIGH);
            for i in range(0,self.width):#for(int i=0;i<self.width; i++)
                if(self.Device == Device_SPI):
                    spi_writebyte([~pBuf[i+self.width*page]]); 
                else :
                    i2c_writebyte(0x40, ~pBuf[i+self.width*page])

    def clear(self):
        """Clear contents of image buffer"""
        _buffer = [0xff]*(self.width * self.height//8)
        self.ShowImage(_buffer) 
            #print "%d",_buffer[i:i+4096]

if __name__ == '__main__':
    disp = SH1106()
    disp.Init()
    disp.clear()

    font10 = ImageFont.truetype('./Font.ttf',13)
    
    while True:
        for line in sys.stdin:
            text = line.strip()
            if text == "PROIVDER_DISPLAY_CLEAR":
                disp.clear()
                continue
            else:
                image = Image.new('1', (disp.width, disp.height), "WHITE")
                draw = ImageDraw.Draw(image)

                draw.text((5,0), text, font = font10, fill = 0)
                disp.ShowImage(disp.getbuffer(image))
                
