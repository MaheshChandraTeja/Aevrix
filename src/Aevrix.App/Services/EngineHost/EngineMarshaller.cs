using System;
using System.Buffers;
using System.Runtime.InteropServices;

namespace Hummingbird.App.Services.EngineHost;




internal static class EngineMarshaller
{
    
    
    
    
    public ref struct Utf8Scoped
    {
        private Span<byte> _buffer;
        private GCHandle _pin;
        private bool _pinned;
        public IntPtr Ptr { get; private set; }

        public Utf8Scoped(ReadOnlySpan<char> text, Span<byte> scratch)
        {
            _buffer = scratch;
            _pin = default;
            _pinned = false;
            Ptr = IntPtr.Zero;

            if (text.IsEmpty)
            {
                if (_buffer.Length > 0) _buffer[0] = 0;
                return;
            }

            var needed = System.Text.Encoding.UTF8.GetByteCount(text) + 1;
            if (needed <= _buffer.Length)
            {
                var written = System.Text.Encoding.UTF8.GetBytes(text, _buffer);
                _buffer[written] = 0;
                _pin = GCHandle.Alloc(_buffer.ToArray(), GCHandleType.Pinned);
                _pinned = true;
                Ptr = _pin.AddrOfPinnedObject();
            }
            else
            {
                var rented = ArrayPool<byte>.Shared.Rent(needed);
                var written = System.Text.Encoding.UTF8.GetBytes(text, rented);
                rented[written] = 0;
                _pin = GCHandle.Alloc(rented, GCHandleType.Pinned);
                _pinned = true;
                Ptr = _pin.AddrOfPinnedObject();
                _buffer = rented;
            }
        }

        public void Dispose()
        {
            if (_pinned)
            {
                _pin.Free();
                
                
                
            }
            Ptr = IntPtr.Zero;
        }
    }
}
