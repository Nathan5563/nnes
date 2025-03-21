use crate::nnes::NNES;

/**
 * Examples:
 * C000  4C F5 C5  JMP $C5F5                       A:00 X:00 Y:00 P:24 SP:FD
 * D0BD  61 80     ADC ($80,X) @ 80 = 0200 = 80    A:7F X:00 Y:63 P:64 SP:FB
 * D0B7  8D 00 02  STA $0200 = 7F                  A:80 X:00 Y:63 P:E5 SP:FB
 * F96E  60        RTS                             A:FF X:00 Y:6E P:27 SP:F9
 */
pub fn trace(nnes: &mut NNES) {
    // TODO: log the current cpu instruction in the above format
}