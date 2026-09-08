# Transporte seguro

## Desenvolvimento VPS

`api-server` usa `127.0.0.1:8080` por padrão. Bind não-loopback falha fechado, exceto quando `OPENIEM_ALLOW_INSECURE_HTTP=true` está definido para desenvolvimento isolado em localhost/LAN. Nunca expor esse modo na Internet.

## Raspberry Pi 5 / produção

Terminar TLS em Caddy ou Nginx na LAN. Proxy encaminha somente para `127.0.0.1:8080`; serviço não deve receber tráfego externo direto.

Exemplo Caddy:

```caddyfile
iem.local {
    reverse_proxy 127.0.0.1:8080
}
```

Certificado deve ser válido para clientes da LAN. Exposição fora da LAN exige TLS válido, autenticação e regras de firewall; HTTP sem proxy é bloqueado pela configuração do serviço.

Execução de PipeWire/ALSA e validação de latência seguem **SIMULATED** no VPS. Só Raspberry Pi 5 confirma hardware real.
