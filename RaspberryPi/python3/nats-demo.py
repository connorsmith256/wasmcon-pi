#!/usr/bin/python
# -*- coding:utf-8 -*-

import SH1106
import time
import config
import traceback

import asyncio
import nats
# from nats.errors import ConnectionClosedError, TimeoutError, NoServersError

import msgpack
import json

from PIL import Image,ImageDraw,ImageFont

async def message_handler(msg):
    subject = msg.subject
    reply = msg.reply
    data = msg.data.decode()
    print("Received a message on '{subject} {reply}': {data}".format(
        subject=subject, reply=reply, data=data))

def draw_text(disp, font, text):
    image = Image.new('1', (disp.width, disp.height), "WHITE")
    draw = ImageDraw.Draw(image)

    # rectangle
    draw.line([(0,0),(127,0)], fill = 0)
    draw.line([(0,0),(0,63)], fill = 0)
    draw.line([(0,63),(127,63)], fill = 0)
    draw.line([(127,0),(127,63)], fill = 0)

    # draw
    print (f"***drawing {text}")
    draw.text((5,0), text, font = font, fill = 0)
    disp.ShowImage(disp.getbuffer(image))

async def main():
    try:
        disp = SH1106.SH1106()

        print("\r\1.3inch OLED init")
        # Initialize library.
        disp.Init()
        # Clear display.
        disp.clear()

        # Create blank image for drawing.
        # image1 = Image.new('1', (disp.width, disp.height), "WHITE")
        # draw = ImageDraw.Draw(image1)
        # font = ImageFont.truetype('Font.ttf', 20)
        font10 = ImageFont.truetype('Font.ttf',13)
        draw_text(disp, font10, "init")

        # image1=image1.rotate(180) 
        # disp.ShowImage(disp.getbuffer(image1))
        # time.sleep(2)
        
        # print ("***draw image")
        # Himage2 = Image.new('1', (disp.width, disp.height), 255)  # 255: clear the frame
        # bmp = Image.open('logo3.bmp')
        # Himage2.paste(bmp, (0,5))
        # # Himage2=Himage2.rotate(180) 	
        # disp.ShowImage(disp.getbuffer(Himage2))

        nc = await nats.connect("nats://127.0.0.1:4222")
        print ("***connected to nats")
        sub = await nc.subscribe("cosmo.hnb.89a6edb9-3644-4a32-88b8-f6f0b1462539.MBCFOPM6JW2APJLXJD3Z5O4CN7CPYJ2B4FTKLJUR5YR5MITIU7HD3WD5.VD7EAHNS6X4PQDN5YQXMZQDPTXODBNEQW52ZTJXM3OAMTHONZPTPJE2U.default")

        try:
            while True:
                msg = await sub.next_msg(timeout=100.0)
                print(f"Received a message on '{msg.subject} {msg.reply}'")
                raw_body = msgpack.unpackb(msg.data)
                # parsed_body = json.loads(raw_body)
                path = raw_body["path"]
                query = raw_body["queryString"]
                sss = f"{path}{query}"
                print(f"Body: {path}{query}")
                draw_text(disp, font10, sss)
        except Exception as e:
            print(f"Exception: {e}")
            pass

        print("done with nats")

    except IOError as e:
        print(e)
        
    except KeyboardInterrupt:    
        print("ctrl + c:")
        epdconfig.module_exit()
        await sub.unsubscribe()
        await nc.drain()
        exit()

if __name__ == '__main__':
    asyncio.run(main())