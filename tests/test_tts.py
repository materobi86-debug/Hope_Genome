"""
Tests for Edge-TTS voice reader and Launchpad equalizer synchronization module.
"""

import pytest
import asyncio
import threading
import time
from launchpad_zcode.tts_sync import speak_text_with_launchpad_sync
from launchpad_zcode.daemon import LaunchpadDaemon

@pytest.mark.asyncio
async def test_tts_launchpad_sync():
    port = 9885
    daemon = LaunchpadDaemon(port=port, virtual=True)
    daemon_thread = threading.Thread(target=daemon.start, daemon=True)
    daemon_thread.start()
    time.sleep(0.1)

    # Test short sentence speaking with equalizer sync
    await speak_text_with_launchpad_sync("Szia Jules! Ez egy teszt mondat.", host="127.0.0.1", port=port)

    daemon.stop()
