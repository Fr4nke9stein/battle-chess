#!/usr/bin/env python3
"""Сервер мультиплеера «Боевых шахмат» — простой ретранслятор ходов.

Запуск на VPS (нужен только Python 3.8+, без библиотек):

    python3 mp_server.py            # порт 47900
    python3 mp_server.py 5000       # свой порт

Порт должен быть открыт снаружи (например, `ufw allow 47900/tcp`). В игре: меню →
«Мультиплеер» → адрес `IP:порт` → «Подключиться».

Как работает: первые два подключившихся садятся за одну доску — первый играет белыми, второй
чёрными; следующие двое — за следующую. Сервер ничего не считает: он пересылает строки одного
игрока другому. Обе игры делают одни и те же ходы и потому показывают одну позицию.

Строки (текст, по одной на строку):
    сервер → игрок:  WAIT            — ждём второго игрока
                     START white|black — партия началась, ваш цвет
                     LEFT            — соперник отключился
    игрок ↔ игрок:   всё остальное пересылается как есть (ходы — их формат знает игра).
Шифрования нет.
"""

import asyncio
import sys

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 47900
# Длинные строки — не наши: обрываем такое соединение.
MAX_LINE = 4096

waiting = None  # (reader, writer, future) первого игрока, ждущего соперника


def log(*parts):
    print(*parts, flush=True)


async def send(writer, line):
    try:
        writer.write((line + "\n").encode())
        await writer.drain()
    except (ConnectionError, OSError):
        pass


async def relay(reader, writer, name):
    """Пересылает строки от `reader` к `writer`, пока одна из сторон не отвалится."""
    try:
        while True:
            line = await reader.readline()
            if not line or len(line) > MAX_LINE:
                return
            writer.write(line)
            await writer.drain()
    except (ConnectionError, OSError, asyncio.LimitOverrunError, ValueError):
        return
    finally:
        log(name, "отключился")


async def play(first, second):
    (r1, w1), (r2, w2) = first, second
    await send(w1, "START white")
    await send(w2, "START black")
    log("партия:", peer(w1), "(белые) против", peer(w2), "(чёрные)")
    a = asyncio.create_task(relay(r1, w2, peer(w1)))
    b = asyncio.create_task(relay(r2, w1, peer(w2)))
    done, pending = await asyncio.wait({a, b}, return_when=asyncio.FIRST_COMPLETED)
    for task in pending:
        task.cancel()
    # Кто остался — узнаёт, что соперник ушёл.
    for w in (w1, w2):
        await send(w, "LEFT")
        w.close()


def peer(writer):
    info = writer.get_extra_info("peername")
    return f"{info[0]}:{info[1]}" if info else "?"


async def wait_eof(reader):
    """Ждёт, пока ожидающий игрок отключится (обрыв — тоже)."""
    try:
        await reader.read(1)
    except (ConnectionError, OSError):
        pass


async def on_connect(reader, writer):
    global waiting
    log("подключился", peer(writer))
    if waiting is not None and not waiting[2].done():
        # Второй игрок: отдаём его ждущему, партию ведёт обработчик первого.
        waiting[2].set_result((reader, writer))
        waiting = None
        return
    paired = asyncio.get_running_loop().create_future()
    waiting = (reader, writer, paired)
    await send(writer, "WAIT")
    # До START игрок ничего не шлёт: конец потока — значит, ушёл, не дождавшись.
    gone = asyncio.create_task(wait_eof(reader))
    await asyncio.wait({paired, gone}, return_when=asyncio.FIRST_COMPLETED)
    if not paired.done():
        paired.cancel()
        if waiting is not None and waiting[2] is paired:
            waiting = None
        log(peer(writer), "ушёл, не дождавшись соперника")
        writer.close()
        return
    gone.cancel()
    await play((reader, writer), paired.result())


async def main():
    server = await asyncio.start_server(on_connect, "0.0.0.0", PORT, limit=MAX_LINE)
    log(f"Сервер «Боевых шахмат» слушает порт {PORT}. Остановить — Ctrl+C.")
    async with server:
        await server.serve_forever()


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        pass
