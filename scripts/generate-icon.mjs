import { deflateSync } from 'node:zlib'
import { writeFileSync, mkdirSync } from 'node:fs'

const size = 32
const rows = []
for (let y = 0; y < size; y += 1) {
  const row = Buffer.alloc(1 + size * 4)
  const center = 15.5
  for (let x = 0; x < size; x += 1) {
    const distance = Math.hypot(x - center, y - center)
    const offset = 1 + x * 4
    if (distance <= 11.5) row.set([36, 184, 112, 255], offset)
  }
  rows.push(row)
}

function chunk(type, data) {
  const typeBuffer = Buffer.from(type)
  const crc = Buffer.alloc(4)
  crc.writeUInt32BE(crc32(Buffer.concat([typeBuffer, data])), 0)
  const length = Buffer.alloc(4)
  length.writeUInt32BE(data.length, 0)
  return Buffer.concat([length, typeBuffer, data, crc])
}

function crc32(buffer) {
  let crc = 0xffffffff
  for (const byte of buffer) {
    crc ^= byte
    for (let bit = 0; bit < 8; bit += 1) crc = (crc >>> 1) ^ (0xedb88320 & -(crc & 1))
  }
  return (crc ^ 0xffffffff) >>> 0
}

const header = Buffer.from('\x89PNG\r\n\x1a\n', 'binary')
const ihdr = Buffer.alloc(13)
ihdr.writeUInt32BE(size, 0)
ihdr.writeUInt32BE(size, 4)
ihdr[8] = 8
ihdr[9] = 6
const png = Buffer.concat([header, chunk('IHDR', ihdr), chunk('IDAT', deflateSync(Buffer.concat(rows))), chunk('IEND', Buffer.alloc(0))])
mkdirSync('src-tauri/icons', { recursive: true })
writeFileSync('src-tauri/icons/icon.png', png)

const icoHeader = Buffer.alloc(6)
icoHeader.writeUInt16LE(0, 0)
icoHeader.writeUInt16LE(1, 2)
icoHeader.writeUInt16LE(1, 4)
const icoEntry = Buffer.alloc(16)
icoEntry[0] = size
icoEntry[1] = size
icoEntry[2] = 0
icoEntry[3] = 0
icoEntry.writeUInt16LE(1, 4)
icoEntry.writeUInt16LE(32, 6)
icoEntry.writeUInt32LE(png.length, 8)
icoEntry.writeUInt32LE(22, 12)
writeFileSync('src-tauri/icons/icon.ico', Buffer.concat([icoHeader, icoEntry, png]))
