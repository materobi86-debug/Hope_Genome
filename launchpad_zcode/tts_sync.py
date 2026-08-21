"""
Edge-TTS Noémi Voice Reader with Real-Time Launchpad Mini MK3 8-Bar Equalizer Sync.
"""

import asyncio
import argparse
import sys
import os
import tempfile
import re
import subprocess
from launchpad_zcode.cli import send_notification


VOICE_HUNGARIAN_NOEMI = "hu-HU-NoemiNeural"


async def speak_text_with_launchpad_sync(text: str, host: str = "127.0.0.1", port: int = 9876):
    """
    Splits input text into sentences and speaks them sequentially using Edge-TTS Noémi voice,
    while streaming real-time 8-bar audio spectrum visualizer events to Launchpad Mini MK3.
    """
    import edge_tts

    # Split into sentences
    sentences = [s.strip() for s in re.split(r'(?<=[.!?])\s+', text) if s.strip()]
    if not sentences:
        sentences = [text]

    print(f"[TTS Noémi] Felolvasás indítása ({len(sentences)} mondat)...")

    for idx, sentence in enumerate(sentences):
        print(f"[TTS Noémi] ({idx+1}/{len(sentences)}): \"{sentence}\"")

        # Trigger real-time audio spectrum equalizer animation on Launchpad
        send_notification("command", animation="equalizer_bars", host=host, port=port)

        temp_dir = tempfile.gettempdir()
        output_file = os.path.join(temp_dir, f"tts_sentence_{idx}.mp3")
        communicate = edge_tts.Communicate(sentence, VOICE_HUNGARIAN_NOEMI)
        await communicate.save(output_file)

        # Play audio on Windows/Linux/macOS using OS media player or ffplay/wmplayer
        play_proc = None
        if os.name == 'nt':
            # Windows PowerShell MediaPlayer for MP3 files
            vbs_cmd = f"Add-Type -AssemblyName presentationCore; $player = New-Object System.Windows.Media.MediaPlayer; $player.Open('{output_file}'); $player.Play(); Start-Sleep -s 3"
            play_proc = subprocess.Popen(["powershell", "-c", vbs_cmd])
        else:
            # Linux/macOS
            for player in ["ffplay", "paplay", "afplay", "mpg123"]:
                if subprocess.call(["which", player], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) == 0:
                    play_proc = subprocess.Popen([player, "-nodisp", "-autoexit", output_file], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                    break

        duration_estimate = max(1.0, len(sentence) * 0.08)
        frames = int(duration_estimate * 10)
        for _ in range(frames):
            send_notification("command", animation="equalizer_bars", host=host, port=port)
            await asyncio.sleep(0.1)

        if play_proc:
            try:
                play_proc.wait(timeout=1.0)
            except Exception:
                pass

        if os.path.exists(output_file):
            try:
                os.remove(output_file)
            except Exception:
                pass

    # Finish with success ripple
    send_notification("task_success", animation="success_ripple", host=host, port=port)
    print("[TTS Noémi] Felolvasás befejezve!")


def main():
    parser = argparse.ArgumentParser(description="Edge-TTS Noémi Voice Reader with Launchpad Equalizer Sync")
    parser.add_argument("text", help="A felolvasandó szöveg")
    parser.add_argument("--host", default="127.0.0.1", help="Daemon host")
    parser.add_argument("--port", type=int, default=9876, help="Daemon port")

    args = parser.parse_args()
    asyncio.run(speak_text_with_launchpad_sync(args.text, host=args.host, port=args.port))


if __name__ == "__main__":
    main()
