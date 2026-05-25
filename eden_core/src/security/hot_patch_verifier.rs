// eden_core/src/security/hot_patch_verifier.rs
// VERIFICADOR DE PARCHES EN CALIENTE - Analiza código máquina para detectar instrucciones peligrosas.
// Este módulo protege contra parches maliciosos que intentarían escapar del sandbox.

use std::io::Read;

///Resultado de verificar un parche.
#[derive(Debug, Clone)]
pub struct ResultadoVerificacion {
    pub seguro: bool,
    pub instrucciones_prohibidas: Vec<InstruccionProhibida>,
    pub saltos_invalidos: Vec<SaltoInvalido>,
    pub mensaje: String,
}

#[derive(Debug, Clone)]
pub struct InstruccionProhibida {
    pub nombre: &'static str,
    pub offset: usize,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SaltoInvalido {
    pub offset: usize,
    pub destino: usize,
    pub longitud_parche: usize,
}

/// Verifica un buffer de código máquina x86_64.
/// Retorna true si es seguro, false si contiene instrucciones prohibidas.
pub fn verificar_codigo_seguro(codigo: &[u8]) -> bool {
    verificar_parche(codigo).seguro
}

/// Verificación completa con reporte detallado.
pub fn verificar_parche(codigo: &[u8]) -> ResultadoVerificacion {
    let mut prohibitions = Vec::new();
    let mut saltos_invalidos = Vec::new();
    let longitud = codigo.len();

    let mut i = 0;
    while i < codigo.len() {
        let byte = codigo[i];
        match byte {
            // INT 0x80 (llamada al sistema legacy de 32 bits)
            0xCD => {
                if i + 1 < codigo.len() && codigo[i + 1] == 0x80 {
                    prohibitions.push(InstruccionProhibida {
                        nombre: "INT 0x80",
                        offset: i,
                        bytes: vec![0xCD, 0x80],
                    });
                    i += 2;
                } else {
                    i += 1;
                }
            }
            // Saltos relativos 32-bit: CALL, JMP
            0xE8 | 0xE9 => {
                if i + 4 < codigo.len() {
                    let offset = i32::from_le_bytes([
                        codigo[i + 1],
                        codigo[i + 2],
                        codigo[i + 3],
                        codigo[i + 4],
                    ]);
                    let destino = (i as isize + 5 + offset as isize) as usize;
                    if destino >= longitud {
                        saltos_invalidos.push(SaltoInvalido {
                            offset: i,
                            destino,
                            longitud_parche: longitud,
                        });
                    }
                }
                i += 5;
            }
            // Salto relativo 8-bit: Jcc short
            0x70..=0x7F => {
                if i + 1 < codigo.len() {
                    let offset = codigo[i + 1] as i8 as isize;
                    let destino = (i as isize + 2 + offset) as usize;
                    if destino >= longitud {
                        saltos_invalidos.push(SaltoInvalido {
                            offset: i,
                            destino,
                            longitud_parche: longitud,
                        });
                    }
                }
                i += 2;
            }
            // Todas las instrucciones 0x0F xx son把它们合并
            0x0F if i + 1 < codigo.len() => {
                match codigo[i + 1] {
                    0x05 => { prohibitions.push(InstruccionProhibida { nombre: "SYSCALL", offset: i, bytes: vec![0x0F, 0x05] }); i += 2; }
                    0x34 => { prohibitions.push(InstruccionProhibida { nombre: "SYSENTER", offset: i, bytes: vec![0x0F, 0x34] }); i += 2; }
                    0x07 => { prohibitions.push(InstruccionProhibida { nombre: "SYSRET", offset: i, bytes: vec![0x0F, 0x07] }); i += 2; }
                    0x01 if i + 2 < codigo.len() && codigo[i + 2] == 0xF9 => { prohibitions.push(InstruccionProhibida { nombre: "RDTSCP", offset: i, bytes: vec![0x0F, 0x01, 0xF9] }); i += 3; }
                    0x01 if i + 2 < codigo.len() && (codigo[i + 2] == 0x02 || codigo[i + 2] == 0x03) => { prohibitions.push(InstruccionProhibida { nombre: "LGDT/LIDT", offset: i, bytes: vec![0x0F, 0x01, codigo[i + 2]] }); i += 3; }
                    0x20 | 0x22 => { prohibitions.push(InstruccionProhibida { nombre: "MOV_CR", offset: i, bytes: vec![0x0F, codigo[i + 1]] }); i += 2; }
                    0x21 | 0x23 => { prohibitions.push(InstruccionProhibida { nombre: "MOV_DR", offset: i, bytes: vec![0x0F, codigo[i + 1]] }); i += 2; }
                    0x31 => { prohibitions.push(InstruccionProhibida { nombre: "RDTSC", offset: i, bytes: vec![0x0F, 0x31] }); i += 2; }
                    0xA2 => { prohibitions.push(InstruccionProhibida { nombre: "CPUID", offset: i, bytes: vec![0x0F, 0xA2] }); i += 2; }
                    _ => i += 2,
                }
            }
            // STI (0xFB), CLI (0xFA)
            0xFB | 0xFA => {
                prohibitions.push(InstruccionProhibida {
                    nombre: if byte == 0xFB { "STI" } else { "CLI" },
                    offset: i,
                    bytes: vec![byte],
                });
                i += 1;
            }
            // IN/OUT (0xE4-0xE7, 0xEC-0xEF)
            0xE4..=0xE7 | 0xEC..=0xEF => {
                prohibitions.push(InstruccionProhibida { nombre: "IN/OUT", offset: i, bytes: vec![byte] });
                i += 1;
            }
            // HLT (0xF4)
            0xF4 => {
                prohibitions.push(InstruccionProhibida { nombre: "HLT", offset: i, bytes: vec![0xF4] });
                i += 1;
            }
            // Todos los demás bytes: avanzamos 1 (no son instrucciones prohibidas conocidas)
            _ => i += 1,
        }
    }

    let seguro = prohibitions.is_empty() && saltos_invalidos.is_empty();
    let mensaje = if seguro {
        String::from("Parche verificado: sin instrucciones prohibidas detectadas.")
    } else {
        let mut msg = String::from("Parche RECHAZADO: ");
        if !prohibitions.is_empty() {
            msg.push_str(&format!("{} instrucciones prohibidas. ", prohibitions.len()));
        }
        if !saltos_invalidos.is_empty() {
            msg.push_str(&format!("{} saltos inválidos. ", saltos_invalidos.len()));
        }
        msg
    };

    ResultadoVerificacion {
        seguro,
        instrucciones_prohibidas: prohibitions,
        saltos_invalidos,
        mensaje,
    }
}

/// Verifica código desde una cadena Base64 (para integrar con Demiurgo Python).
#[allow(dead_code)]
pub fn verificar_parche_base64(base64_codigo: &str) -> ResultadoVerificacion {
    // Implementación simple sin dependencia de base64 crate.
    // Si el input no es ASCII válido para base64, rechazar.
    let es_base64_valido = base64_codigo.bytes().all(|b| {
        b.is_ascii_alphanumeric() || b == b'+' || b == b'/' || b == b'='
    });
    if !es_base64_valido {
        return ResultadoVerificacion {
            seguro: false,
            instrucciones_prohibidas: Vec::new(),
            saltos_invalidos: Vec::new(),
            mensaje: String::from("Error: Base64 inválido"),
        };
    }
    // Por ahora, siempre marcar como inseguro hasta que se implemente la decodificación real.
    ResultadoVerificacion {
        seguro: false,
        instrucciones_prohibidas: Vec::new(),
        saltos_invalidos: Vec::new(),
        mensaje: String::from("Base64 no implementado aún"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vacio_es_seguro() {
        assert!(verificar_codigo_seguro(&[]));
    }

    #[test]
    fn test_syscall_detectado() {
        // 0x0F 0x05 = SYSCALL
        let codigo = [0x0F, 0x05, 0x90, 0x90];
        assert!(!verificar_codigo_seguro(&codigo));
    }

    #[test]
    fn test_sysenter_detectado() {
        // 0x0F 0x34 = SYSENTER
        let codigo = [0x0F, 0x34, 0x90];
        assert!(!verificar_codigo_seguro(&codigo));
    }

    #[test]
    fn test_int_80_detectado() {
        // 0xCD 0x80 = INT 0x80
        let codigo = [0xCD, 0x80, 0x90];
        assert!(!verificar_codigo_seguro(&codigo));
    }

    #[test]
    fn test_nops_son_seguros() {
        let codigo = [0x90, 0x90, 0x90, 0x90];
        assert!(verificar_codigo_seguro(&codigo));
    }

    #[test]
    fn test_salto_fuera_detectado() {
        // JMP near rel32 que salta más allá del tamaño del buffer
        // 0xE9 = JMP rel32, seguido de offset 0xFF 0xFF 0xFF 0x7F (= +2GB, fuera de cualquier parche)
        let codigo = [0xE9, 0xFF, 0xFF, 0xFF, 0x7F, 0x90, 0x90];
        assert!(!verificar_codigo_seguro(&codigo));
    }

    #[test]
    fn test_sti_detectado() {
        let codigo = [0xFB, 0x90];
        assert!(!verificar_codigo_seguro(&codigo));
    }

    #[test]
    fn test_cli_detectado() {
        let codigo = [0xFA, 0x90];
        assert!(!verificar_codigo_seguro(&codigo));
    }

    #[test]
    fn test_hlt_detectado() {
        let codigo = [0xF4, 0x90];
        assert!(!verificar_codigo_seguro(&codigo));
    }

    #[test]
    fn test_resultado_verificacion_detalle() {
        let resultado = verificar_parche(&[0x0F, 0x05]);
        assert!(!resultado.seguro);
        assert_eq!(resultado.instrucciones_prohibidas.len(), 1);
        assert_eq!(resultado.instrucciones_prohibidas[0].nombre, "SYSCALL");
        assert_eq!(resultado.mensaje, "Parche RECHAZADO: 1 instrucciones prohibidas. ");
    }

    #[test]
    fn test_base64_invalido() {
        let resultado = verificar_parche_base64("¡no es base64!");
        assert!(!resultado.seguro);
        assert!(resultado.mensaje.contains("Base64 inválido"));
    }
}
