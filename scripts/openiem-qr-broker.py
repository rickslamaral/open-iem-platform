#!/usr/bin/env python3
"""Minimal local QR capability broker. Secret stays memory-only and never logs."""
import hashlib, json, os, secrets, socketserver, threading, time, urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

TTL_DEFAULT=14400; TTL_MIN=60; TTL_MAX=86400

def ttl():
    try: value=int(os.environ.get('OPENIEM_QR_SESSION_TTL_SECONDS','14400'))
    except ValueError: value=TTL_DEFAULT
    return value if TTL_MIN <= value <= TTL_MAX else TTL_DEFAULT

class Broker:
    def __init__(self):
        self.lock=threading.Lock(); self.secret=None; self.expires=0; self.generation=0
        self.api=os.environ.get('OPENIEM_API_URL','http://127.0.0.1:8080').rstrip('/')
        self.base=os.environ.get('OPENIEM_SESSION_PUBLIC_BASE',self.api).rstrip('/')
        self.cap_path=os.environ.get('OPENIEM_QR_BROKER_CAPABILITY_FILE','/run/openiem-qr-broker/capability')
        self.rotate()
    def rotate(self):
        with self.lock:
            self.secret=secrets.token_hex(32); self.expires=int(time.time())+ttl(); self.generation+=1
            secret=self.secret; expires=self.expires
        try:
            with open(self.cap_path,encoding='utf-8') as f: capability=f.read().strip()
            body=json.dumps({'qr_secret':secret,'expires_at':expires}).encode()
            req=urllib.request.Request(self.api+'/api/v1/internal/qr/broker',body=body,method='POST',headers={'Content-Type':'application/json','X-OpenIEM-QR-Broker-Capability':capability})
            with urllib.request.urlopen(req,timeout=5) as response:
                if response.status != 204: raise RuntimeError('broker registration rejected')
        except Exception:
            with self.lock: self.secret=None; self.expires=0
            raise
    def public(self):
        with self.lock:
            expired = not self.secret or time.time() >= self.expires
        if expired:
            try: self.rotate()
            except Exception: return None
        with self.lock:
            return {'url':self.base+'/musician/#invitation='+self.secret,'expires_at':self.expires,'generation':self.generation}

broker=Broker()
class Handler(BaseHTTPRequestHandler):
    def log_message(self,*args): pass
    def do_GET(self):
        if self.path != '/' and self.path != '/qr': self.send_error(404); return
        payload=broker.public()
        if payload is None: self.send_error(410); return
        # Fragment keeps secret out of HTTP query, Referer, server logs, and browser storage.
        data=('<!doctype html><meta charset="utf-8"><meta name="referrer" content="no-referrer">'
              '<meta http-equiv="Cache-Control" content="no-store"><title>Open IEM QR</title>'
              '<h1>Open IEM musician QR</h1><p>Scan or open this URL:</p>'
              '<a id="url" rel="noreferrer"></a><p id="expiry"></p><script>'
              'const p='+json.dumps(payload,separators=(',',':'))+';'
              'document.getElementById("url").href=p.url;document.getElementById("url").textContent=p.url;'
              'document.getElementById("expiry").textContent="Expires: "+new Date(p.expires_at*1000).toISOString();'
              'history.replaceState(null,"",location.pathname);</script>').encode()
        self.send_response(200); self.send_header('Content-Type','text/html; charset=utf-8'); self.send_header('Cache-Control','no-store'); self.send_header('Referrer-Policy','no-referrer'); self.send_header('X-Content-Type-Options','nosniff'); self.send_header('Content-Security-Policy',"default-src 'none'; script-src 'unsafe-inline'; base-uri 'none'; frame-ancestors 'none'"); self.end_headers(); self.wfile.write(data)

if __name__=='__main__':
    bind=os.environ.get('OPENIEM_QR_BROKER_BIND','0.0.0.0:8090'); host,port=bind.rsplit(':',1)
    ThreadingHTTPServer((host,int(port)),Handler).serve_forever()
