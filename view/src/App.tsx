import 'xterm/css/xterm.css'
import {useEffect, useRef} from "react";
import { Terminal } from 'xterm';
import { AttachAddon } from 'xterm-addon-attach';

function App() {
    const terminalRef = useRef(null);

    useEffect(() => {
        if (!terminalRef.current) return;
        const term = new Terminal();
        term.open(terminalRef.current);
        const ws = new WebSocket('ws://localhost:8080/ws');
        ws.onopen = () => {
            const attachAddon = new AttachAddon(ws);
            term.loadAddon(attachAddon);
        }

        return () => {
            term.dispose();
        }
    }, [])


    return <div
        ref={terminalRef}
        id="terminal"></div>
}


export default App
