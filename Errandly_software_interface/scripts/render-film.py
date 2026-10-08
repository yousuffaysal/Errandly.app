"""Render original 24-second Errandly concept film (Pillow + imageio-ffmpeg)."""
from PIL import Image, ImageDraw, ImageFont
import sys, math, os, textwrap
sys.path.insert(0,'/tmp/errandly-video-deps')
import imageio_ffmpeg
ROOT=os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
W,H=1280,720
font='/System/Library/Fonts/Supplemental/Arial.ttf'
def f(size):return ImageFont.truetype(font,size)
wall=Image.open(ROOT+'/public/fjord.jpg').convert('RGB').resize((W,H))
def frame(t):
 im=wall.copy(); overlay=Image.new('RGBA',(W,H),(13,18,12,115));im=Image.alpha_composite(im.convert('RGBA'),overlay);d=ImageDraw.Draw(im)
 d.text((55,32),'errandly',font=f(27),fill='#efefe6');d.text((920,40),'PRODUCT CONCEPT / COMING SOON',font=f(12),fill='#d5d8cb')
 if t<4 or t>=20:
  title='Your everyday work.' if t<4 else 'Your computer. Your AI. Your data.'
  title2='Handled by Errandly.' if t<4 else 'Errandly for Mac. Coming soon.'
  d.text((80,260),title,font=f(56 if t<4 else 44),fill='#f4f3e9');d.text((80,335),title2,font=f(56 if t<4 else 32),fill='#e1e4d5')
  d.text((82,445),'A little less on your plate.',font=f(22),fill='#bec7b1')
 else:
  section=min(2,int((t-4)//5.34));local=(t-4)%5.34
  titles=['Turn invoices into insights.','Turn documents into understanding.','Turn clutter into clarity.']
  prompts=['Analyze my October invoices and create an expense report.','Summarize these lecture PDFs and prepare study notes.','Organize my Downloads folder. Show me the changes first.']
  steps=[['Read 12 selected invoices','Categorize expenses','Create October_expenses.xlsx'],['Read 5 lecture PDFs','Group ideas by topic','Create Study_notes.md'],['Scan selected files','Prepare 24 proposed moves','Wait for your approval']][section]
  d.text((65,95),titles[section],font=f(34),fill='#f0f1e6')
  d.rounded_rectangle((65,164,1215,627),radius=12,fill='#1e211a',outline='#65705c',width=1)
  for i,c in enumerate(['#d78274','#d3b966','#8ba47a']):d.ellipse((85+i*18,184,94+i*18,193),fill=c)
  d.text((165,180),'Errandly / Personal workspace',font=f(14),fill='#b1baa4')
  d.line((65,211,1215,211),fill='#3a4232')
  d.rounded_rectangle((92,241,628,310),radius=7,fill='#343b2e');d.text((109,257),textwrap.wrap(prompts[section],width=53)[0],font=f(17),fill='#e1e7d8')
  if len(prompts[section])>53:d.text((109,280),textwrap.wrap(prompts[section],width=53)[1],font=f(17),fill='#e1e7d8')
  d.text((97,339),'errandly',font=f(19),fill='#e7ecdf')
  for i,s in enumerate(steps):
   done=local>i+0.5;c='#c5d4b0' if done else '#7f8a73'
   d.ellipse((98,388+i*47,117,407+i*47),outline=c,width=1)
   d.text((130,386+i*47),s,font=f(17),fill=c)
   if done:d.line([(102,397+i*47),(106,401+i*47),(113,391+i*47)],fill=c,width=2)
  d.text((98,569),'On-device AI. Your files stay on this Mac.',font=f(14),fill='#929f82')
  d.rounded_rectangle((666,233,1186,599),radius=5,fill='#e6e8dd')
  d.text((700,267),['October expense report','Your study notes','Proposed file organization'][section],font=f(25),fill='#303a27')
  d.text((700,312),['Total expenses  $1,463.00','5 sources / 30 practice questions','24 files / 4 categories'][section],font=f(20),fill='#71805f')
  rows=[['Software                 $248.00','Workspace              $650.00','Equipment              $379.00','Travel                       $186.00'],['01   Probability foundations','02   Distributions and sampling','03   Statistical inference','04   Practice questions'],['Documents              12 files','Spreadsheets              5 files','Images                         4 files','Archives                      3 files']][section]
  for i,s in enumerate(rows):
   y=363+i*47;d.line((700,y-10,1152,y-10),fill='#c4cdb7');d.text((700,y),s,font=f(18),fill='#4b593d')
 d.text((55,673),'Illustrated workflow. Sample data. Planned product experience.',font=f(12),fill='#d0d8c4')
 d.rectangle((55,701,55+1170*min(t/24,1),703),fill='#e6ecdc')
 return im.convert('RGB')
frame(6.5).save(ROOT+'/public/product-poster.jpg',quality=92)
writer=imageio_ffmpeg.write_frames(ROOT+'/public/errandly-film.mp4',(W,H),fps=24,codec='libx264',quality=7,pix_fmt_out='yuv420p',output_params=['-movflags','+faststart'])
writer.send(None)
for n in range(576):writer.send(frame(n/24).tobytes())
writer.close()
print('Created 24 second film and poster')
