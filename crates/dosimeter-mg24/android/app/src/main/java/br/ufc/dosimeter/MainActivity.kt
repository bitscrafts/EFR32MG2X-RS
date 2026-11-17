package br.ufc.dosimeter

//! Cliente BLE mínimo do dosímetro: procura o dispositivo, liga-se, subscreve as três
//! características e mostra nível, intervalo e dose.
//!
//! ## Porque é pequena
//!
//! Não é um produto: é o cliente que prova que o dispositivo fala o contrato GATT e que os
//! números chegam ao telemóvel. Uma app completa (histórico, gráficos, exportação) é
//! trabalho futuro — o que a disciplina avalia é o modelo, e a app não dá nota.
//!
//! ## Contrato (igual ao de `spec/E1-APP.DOSIMETER.md`, secção E1-BLE.1)
//!
//! ```text
//! serviço       D05E0001-0000-4000-8000-00805F9B34FB
//! nivel         D05E0002-…   f32 LE, dB(A),            notify 1 Hz
//! intervalo     D05E0003-…   2 x f32 LE (min, max), dB, notify 1 Hz
//! dose          D05E0004-…   f32 LE, %,                notify 0,1 Hz
//! comando       D05E0005-…   write: 0x01 = repor dose
//! ```
//!
//! Um `f32` little-endian em vez de `sint16` com escala: o firmware é Rust e o valor já é
//! `f32`; converter duas vezes só cria duas oportunidades de erro.

import android.Manifest
import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothGatt
import android.bluetooth.BluetoothGattCallback
import android.bluetooth.BluetoothGattCharacteristic
import android.bluetooth.BluetoothGattDescriptor
import android.bluetooth.BluetoothManager
import android.bluetooth.BluetoothProfile
import android.bluetooth.le.ScanCallback
import android.bluetooth.le.ScanResult
import android.bluetooth.le.ScanSettings
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import android.widget.TextView
import androidx.activity.ComponentActivity
import androidx.core.app.ActivityCompat
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.util.UUID

private val SERVICO: UUID = UUID.fromString("D05E0001-0000-4000-8000-00805F9B34FB")
private val CHAR_NIVEL: UUID = UUID.fromString("D05E0002-0000-4000-8000-00805F9B34FB")
private val CHAR_INTERVALO: UUID = UUID.fromString("D05E0003-0000-4000-8000-00805F9B34FB")
private val CHAR_DOSE: UUID = UUID.fromString("D05E0004-0000-4000-8000-00805F9B34FB")
private val CHAR_COMANDO: UUID = UUID.fromString("D05E0005-0000-4000-8000-00805F9B34FB")
private val CCCD: UUID = UUID.fromString("00002902-0000-1000-8000-00805F9B34FB")

class MainActivity : ComponentActivity() {

    private lateinit var visor: TextView
    private var gatt: BluetoothGatt? = null

    private var nivelDb: Float? = null
    private var intervalo: Pair<Float, Float>? = null
    private var dosePct: Float? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        visor = TextView(this).apply {
            textSize = 20f
            setPadding(48, 48, 48, 48)
            text = getString(R.string.a_procurar)
        }
        setContentView(visor)
        pedirPermissoes()
    }

    private fun pedirPermissoes() {
        val precisa = mutableListOf<String>()
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            precisa += Manifest.permission.BLUETOOTH_SCAN
            precisa += Manifest.permission.BLUETOOTH_CONNECT
        } else {
            precisa += Manifest.permission.ACCESS_FINE_LOCATION
        }
        val porPedir = precisa.filter {
            ActivityCompat.checkSelfPermission(this, it) != PackageManager.PERMISSION_GRANTED
        }
        if (porPedir.isEmpty()) {
            procurar()
        } else {
            ActivityCompat.requestPermissions(this, porPedir.toTypedArray(), 1)
        }
    }

    override fun onRequestPermissionsResult(
        codigo: Int,
        permissoes: Array<String>,
        resultados: IntArray,
    ) {
        super.onRequestPermissionsResult(codigo, permissoes, resultados)
        if (codigo == 1 && resultados.all { it == PackageManager.PERMISSION_GRANTED }) {
            procurar()
        } else {
            visor.text = getString(R.string.sem_permissoes)
        }
    }

    private fun procurar() {
        val manager = getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager
        val adaptador: BluetoothAdapter = manager.adapter ?: run {
            visor.text = getString(R.string.sem_bluetooth)
            return
        }
        if (!adaptador.isEnabled) {
            visor.text = getString(R.string.ligue_bluetooth)
            return
        }
        val scanner = adaptador.bluetoothLeScanner ?: return
        val definicoes = ScanSettings.Builder()
            .setScanMode(ScanSettings.SCAN_MODE_LOW_LATENCY)
            .build()

        val callback = object : ScanCallback() {
            override fun onScanResult(tipo: Int, resultado: ScanResult) {
                scanner.stopScan(this)
                visor.text = getString(R.string.a_ligar, resultado.device.address)
                gatt = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                    resultado.device.connectGatt(
                        this@MainActivity, false, gattCallback, BluetoothDevice_TRANSPORT_LE
                    )
                } else {
                    resultado.device.connectGatt(this@MainActivity, false, gattCallback)
                }
            }

            override fun onScanFailed(erro: Int) {
                visor.text = getString(R.string.falha_procura, erro)
            }
        }
        visor.text = getString(R.string.a_procurar)
        scanner.startScan(null, definicoes, callback)
    }

    private val gattCallback = object : BluetoothGattCallback() {

        override fun onConnectionStateChange(g: BluetoothGatt, estado: Int, novo: Int) {
            if (novo == BluetoothProfile.STATE_CONNECTED) {
                g.discoverServices()
            } else {
                visor.text = getString(R.string.desligado)
            }
        }

        override fun onServicesDiscovered(g: BluetoothGatt, estado: Int) {
            val servico = g.getService(SERVICO) ?: run {
                visor.text = getString(R.string.servico_ausente)
                return
            }
            for (uuid in listOf(CHAR_NIVEL, CHAR_INTERVALO, CHAR_DOSE)) {
                val c = servico.getCharacteristic(uuid) ?: continue
                g.setCharacteristicNotification(c, true)
                c.getDescriptor(CCCD)?.let { d ->
                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                        g.writeDescriptor(d, BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE)
                    } else {
                        @Suppress("DEPRECATION")
                        d.value = BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE
                        @Suppress("DEPRECATION")
                        g.writeDescriptor(d)
                    }
                }
            }
            visor.text = getString(R.string.ligado)
        }

        override fun onCharacteristicChanged(
            g: BluetoothGatt,
            c: BluetoothGattCharacteristic,
            valor: ByteArray,
        ) {
            quandoChega(c.uuid, valor)
        }

        @Deprecated("API < 33")
        override fun onCharacteristicChanged(g: BluetoothGatt, c: BluetoothGattCharacteristic) {
            @Suppress("DEPRECATION")
            quandoChega(c.uuid, c.value ?: return)
        }
    }

    /** Um único ponto de parsing, com o tamanho verificado antes de ler. */
    private fun quandoChega(uuid: UUID, valor: ByteArray) {
        val buf = ByteBuffer.wrap(valor).order(ByteOrder.LITTLE_ENDIAN)
        when (uuid) {
            CHAR_NIVEL -> if (valor.size >= 4) nivelDb = buf.float
            CHAR_INTERVALO -> if (valor.size >= 8) intervalo = buf.float to buf.float
            CHAR_DOSE -> if (valor.size >= 4) dosePct = buf.float
        }
        mostrar()
    }

    private fun mostrar() {
        val nivel = nivelDb?.let { n ->
            val iv = intervalo
            if (iv == null) "%.1f dB".format(n)
            else "%.1f dB  [%.1f … %.1f]".format(n, iv.first, iv.second)
        } ?: "—"
        val dose = dosePct?.let { "%.1f %%".format(it) } ?: "—"
        visor.text = getString(R.string.painel, nivel, dose)
    }

    override fun onDestroy() {
        super.onDestroy()
        gatt?.close()
    }
}

/** Transporte LE explícito (evita falhas em telemóveis com transport autodetectado). */
private const val BluetoothDevice_TRANSPORT_LE = 2
