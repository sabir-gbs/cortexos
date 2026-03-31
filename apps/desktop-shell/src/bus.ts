/**
 * WebSocket client for the CortexOS command bus.
 *
 * Provides real-time event streaming with automatic reconnection
 * and exponential backoff.
 */

export type WsEventHandler = (event: string, payload: unknown) => void;

export class BusClient {
  private ws: WebSocket | null = null;
  private url: string;
  private handler: WsEventHandler | null = null;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private reconnectDelay = 1000;
  private maxReconnectDelay = 30_000;
  private disposed = false;
  private connected = false;
  private onConnectionChange?: (connected: boolean) => void;

  constructor(baseUrl?: string) {
    const base =
      baseUrl ||
      (typeof location !== "undefined" ? `ws://${location.host}` : "ws://localhost:3000");
    this.url = `${base}/ws`;
  }

  /** Set the event handler. */
  onEvent(handler: WsEventHandler): void {
    this.handler = handler;
  }

  /** Set connection state change callback. */
  setOnConnectionChange(cb: (connected: boolean) => void): void {
    this.onConnectionChange = cb;
  }

  /** Connect to the WebSocket endpoint. Auth is handled via HttpOnly cookies sent automatically by the browser. */
  connect(): void {
    if (this.disposed) return;

    try {
      this.ws = new WebSocket(this.url);
    } catch (err) {
      console.error("WebSocket connection failed:", err);
      this.scheduleReconnect();
      return;
    }

    this.ws.onopen = () => {
      this.connected = true;
      this.reconnectDelay = 1000;
      this.onConnectionChange?.(true);
    };

    this.ws.onclose = () => {
      const wasConnected = this.connected;
      this.connected = false;
      this.ws = null;
      this.onConnectionChange?.(false);
      if (wasConnected) {
        this.handler?.("connection.lost", {});
      }
      this.scheduleReconnect();
    };

    this.ws.onerror = () => {
      // onclose will fire after onerror
    };

    this.ws.onmessage = (event: MessageEvent) => {
      try {
        const frame = JSON.parse(event.data as string);
        if (frame.type && frame.payload !== undefined) {
          this.handler?.(frame.type, frame.payload);
        } else if (frame.event) {
          // Alternative frame format
          this.handler?.(frame.event, frame.data || frame.payload);
        }
      } catch (err) {
        console.warn("Malformed WebSocket message:", err);
      }
    };
  }

  /** Send a command on the bus. */
  send(command: string, payload: unknown): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: command, payload }));
    }
  }

  /** Check if connected. */
  isConnected(): boolean {
    return this.connected;
  }

  /** Disconnect and clean up. */
  dispose(): void {
    this.disposed = true;
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.ws) {
      this.ws.onclose = null;
      this.ws.close();
      this.ws = null;
    }
    this.connected = false;
  }

  private scheduleReconnect(): void {
    if (this.disposed) return;
    if (this.reconnectTimer) return;

    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.connect();
    }, this.reconnectDelay);

    this.reconnectDelay = Math.min(this.reconnectDelay * 2, this.maxReconnectDelay);
  }
}
