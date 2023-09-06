// waveshareoled.smithy

// Tell the code generator how to reference symbols defined in this namespace
metadata package = [ { namespace: "org.cosmonic.wasmcon.waveshareoled", crate: "waveshareoled_interface" } ]

namespace org.cosmonic.wasmcon.waveshareoled

use org.wasmcloud.model#wasmbus
use org.wasmcloud.model#U32
use org.wasmcloud.model#U64

/// The Waveshareoled service has a single method, calculate, which
/// calculates the factorial of its whole number parameter.
@wasmbus(
    contractId: "cosmonic:waveshareoled",
    providerReceive: true )
service Waveshareoled {
  version: "0.1",
  operations: [ DrawMessage, Clear ]
}

@wasmbus(
    contractId: "cosmonic:waveshareoled",
    actorReceive: true)
service WaveshareSubscriber {
  version: "0.1",
  operations: [ HandleEvent ]
}

/// Draws the given message on the OLED display
operation DrawMessage {
  input: DrawMessageInput,
}

structure DrawMessageInput {
  @required
  message: String,
}

operation Clear {}

operation HandleEvent {
  input: Event,
}

string Event
