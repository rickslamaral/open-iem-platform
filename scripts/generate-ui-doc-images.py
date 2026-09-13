from pathlib import Path
import html, subprocess

RSVG_CONVERT = Path('/usr/bin/rsvg-convert')
OUT = Path(__file__).parents[1] / 'docs/images'
OUT.mkdir(parents=True, exist_ok=True)
W, H = 1600, 1000

def esc(s): return html.escape(str(s))
def text(x,y,s,cls='body',fill=None):
    f=f' fill="{fill}"' if fill else ''
    return f'<text x="{x}" y="{y}" class="{cls}"{f}>{esc(s)}</text>'
def rect(x,y,w,h,cls='panel',rx=12): return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{rx}" class="{cls}"/>'
def base(title,subtitle):
    return f'''<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">
<rect width="1600" height="1000" fill="#081426"/><rect width="1600" height="10" fill="#27c9b8"/>
<style>.title{{font:700 38px Arial;fill:#f1f6ff}}.sub{{font:18px Arial;fill:#a9bad1}}.h{{font:700 25px Arial;fill:#f1f6ff}}.body{{font:16px Arial;fill:#d8e4f5}}.muted{{font:15px Arial;fill:#9db0c8}}.mono{{font:16px monospace;fill:#e5eeff}}.panel{{fill:#10233d;stroke:#2c4a70;stroke-width:2}}.control{{fill:#172f52;stroke:#668fff;stroke-width:2}}.active{{fill:#163e42;stroke:#26c9b5;stroke-width:2}}.warn{{fill:#3d3020;stroke:#d9a441;stroke-width:2}}.danger{{fill:#432733;stroke:#e5778d;stroke-width:2}}.purple{{fill:#392852;stroke:#c47aff;stroke-width:2}}.bar{{fill:#263f62}}.fill{{fill:#30c9b6}}</style>
{text(65,68,'Open IEM Platform', 'title')}{text(65,103,title,'h')}{text(65,134,subtitle,'sub')}'''
def end(): return '</svg>\n'

def musician():
 s=base('Musician PWA — interface de usuário','Mockup documental baseado em web/musician/src · 8 canais · controle do próprio mix')
 s+=rect(55,170,1490,760); s+=text(90,215,'Open IEM','h')+text(1190,215,'Conectado · rev 12','body','#36d3bf')+text(1390,215,'Logout','body','#ff9baa')
 s+=rect(90,245,1390,92,'active')+text(120,282,'MASTER VOLUME','muted')+text(120,311,'-3.0 dB','h')+rect(330,282,1050,10,'bar',5)+rect(330,282,820,10,'fill',5)
 names=['Vocal','Guitar','Bass','Keys','Drums L','Drums R','Click','Aux']
 for i,n in enumerate(names):
  x=90+(i%4)*345; y=370+(i//4)*245
  s+=rect(x,y,315,195,'control'); s+=text(x+22,y+35,f'CH{i+1:02d}','muted')+text(x+22,y+68,n,'h')
  s+=text(x+22,y+106,'-6.0 dB','mono')+rect(x+22,y+130,270,9,'bar',5)+rect(x+22,y+130,170,9,'fill',5)
  s+=rect(x+210,y+153,80,28,'danger' if i==5 else 'panel',6)+text(x+229,y+173,'MUTED' if i==5 else 'MUTE','muted')
 s+=text(90,900,'Controles: ganho por canal · mute · volume master · status WebSocket','muted')+end(); return s

def engineer():
 s=base('Engineer Console — mixer e operação','Mockup documental baseado em web/engineer/src · acesso Engineer/Admin')
 s+=rect(55,170,1490,760); s+=text(90,215,'OPEN IEM / CONTROL PLANE','muted')+text(90,255,'Engineer Console','h')+text(1370,250,'Sair','body','#ff9baa')
 s+=rect(90,280,1390,58,'warn')+text(115,316,'Áudio SIMULATED — VPS sem PipeWire; sessões WebRTC não representam mídia validada em hardware.','body','#f0c971')
 metrics=[('Revision','12'),('Sessões ativas','2'),('Backend','simulated'),('XRUNs','UNKNOWN')]
 for i,(a,b) in enumerate(metrics):
  x=90+i*345; s+=rect(x,370,315,105); s+=text(x+20,405,a,'muted')+text(x+20,448,b,'h')
 s+=rect(90,520,680,335); s+=text(120,560,'Mix assignments','h')+text(120,590,'Atribuição exige ID do usuário.','muted')
 for i,(mix,user) in enumerate([('Mix 1','cantor (ID 7)'),('Mix 2','Livre')]):
  y=635+i*90; s+=text(125,y,mix,'body')+text(125,y+27,user,'muted'); s+=rect(550,y-25,170,38,'danger' if i==0 else 'active',6)+text(580,y, 'Remover' if i==0 else 'ID usuário','muted')
 s+=rect(810,520,670,335); s+=text(840,560,'Sessões de áudio','h')+text(840,600,'musician-7','body')+text(1380,600,'ATIVA','body','#36d3bf')+text(840,665,'musician-12','body')+text(1380,665,'ATIVA','body','#36d3bf')+text(840,760,'Atualização periódica: 5 s','muted')+end(); return s

def admin():
 s=base('Admin — CLI e API, sem painel web','Mockup documental: o repositório não possui interface Admin web; comandos reais estão em server/admin-cli')
 s+=rect(55,170,1490,760); s+=text(90,215,'ADMIN OPERATIONS','h')+rect(90,250,1390,100,'panel')+text(120,290,'$ open-iem-admin health','mono')+text(120,325,'{"status":"ok"}','body','#36d3bf')
 s+=rect(90,390,1390,430,'purple'); s+=text(120,435,'Comandos suportados','h')
 cmds=[('health','GET /api/v1/health'),('user list','GET /api/v1/admin/users'),('user create','POST /api/v1/admin/users'),('user delete --id 7','DELETE /api/v1/admin/users/7'),('session list','GET /api/v1/admin/sessions'),('session revoke --id 12','DELETE /api/v1/admin/sessions/12')]
 for i,(a,b) in enumerate(cmds): y=490+i*48; s+=text(135,y,'open-iem-admin '+a,'mono')+text(720,y,b,'body')
 s+=rect(90,855,1390,45,'warn')+text(120,884,'ADMIN: gerenciamento de usuários e sessões · sem criação via Engineer UI','body','#f0c971')+end(); return s

def controls():
 s=base('Controles de mix — ganho, mute e pan','Referência visual dos controles documentados; áudio no VPS permanece SIMULATED')
 s+=rect(55,170,1490,760)
 labels=[('GAIN','-6.0 dB','PUT /api/v1/channels/{index}/gain'),('MUTE','OFF','PUT /api/v1/channels/{index}/mute'),('PAN','L 20','PUT /api/v1/mixes/{mix_idx}/sends/{ch_idx}/pan'),('AUDIO','SIMULATED','POST /api/v1/audio/offer')]
 for i,(a,b,c) in enumerate(labels):
  x=95+i*350; s+=rect(x,250,310,270,'control'); s+=text(x+25,295,a,'h')+text(x+25,355,b,'h','#36d3bf' if a!='AUDIO' else '#f0c971')+rect(x+25,400,260,10,'bar',5)+rect(x+25,400,150 if i<3 else 260,10,'fill',5)+text(x+25,480,c,'muted')
 s+=text(95,630,'WebSocket de controle','h')+rect(95,665,1380,85,'purple')+text(125,710,'openiem.v1  ·  SetChannelGain  ·  SetChannelMute  ·  token no subprotocolo','mono')+text(95,830,'Ownership: músico acessa somente mix atribuído; Engineer/Admin operam mix válidos.','muted')+end(); return s

def login():
 s=base('Login — Engineer Console','Mockup documental baseado nos componentes de autenticação')
 s+=rect(480,230,640,510,'panel'); s+=text(535,300,'OPEN IEM / CONTROL PLANE','muted')+text(535,360,'Engineer Console','h')+text(535,405,'Acesso restrito a Engineer ou Admin.','body')
 for y,label in [(475,'Usuário'),(570,'Senha')]: s+=text(535,y,label,'muted')+rect(535,y+18,530,48,'control',6)
 s+=rect(535,665,530,48,'active',6)+text(755,696,'Entrar','body')+end(); return s

images={'open-iem-musician-ui':musician(),'open-iem-engineer-console':engineer(),'open-iem-admin-cli':admin(),'open-iem-mix-controls':controls(),'open-iem-login':login()}
if not RSVG_CONVERT.is_file():
 raise SystemExit(f'converter not found: {RSVG_CONVERT}')

for name,svg in images.items():
 (OUT/(name+'.svg')).write_text(svg)
 subprocess.run([str(RSVG_CONVERT),str(OUT/(name+'.svg')),'-o',str(OUT/(name+'.png'))],check=True)
 print(name)
