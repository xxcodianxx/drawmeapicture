#include <Arduino.h>
#include <SPI.h>
#include <CRC32.h>
#include <screen.h>

uint16_t w = 0;
uint16_t h = 0;

#define MESSAGE_SIZE 10

void setup()
{
  Serial.begin(9600);

  screen_init();

  while (!Serial);

  Serial.print("HELLO");

  w = tft.width();
  h = tft.height();

  Serial.write((uint8_t)(w >> 8));
  Serial.write((uint8_t)(w));
  Serial.write((uint8_t)(h >> 8));
  Serial.write((uint8_t)(h));
}

struct DrawCall
{
  uint16_t x, y;
  uint16_t color;
};

bool validate_crc32(uint8_t *buf) {
  uint32_t expected = CRC32::calculate(buf, 6);
  uint32_t real = ((uint32_t)buf[MESSAGE_SIZE - 1] << 24) | ((uint32_t)buf[MESSAGE_SIZE - 2] << 16) | ((uint32_t)buf[MESSAGE_SIZE - 3] << 8) | (uint32_t)buf[MESSAGE_SIZE - 4];

  if (expected != real) {
    tft.println("crc32 mismatch!");
    tft.println("expected: " + String(expected, HEX));
    tft.println("    real: " + String(real, HEX));
    return false;
  }

  return true;
}

bool recv_drawcall(uint8_t *buf, DrawCall *pix_ptr)
{
  if (!validate_crc32(buf)) {
    return false;
  }

  memcpy(&pix_ptr->x, buf, 2);
  memcpy(&pix_ptr->y, buf + 2, 2);
  memcpy(&pix_ptr->color, buf + 4, 2);

  return true;
}

void loop()
{
  DrawCall call;

  uint8_t *bytes = new uint8_t[MESSAGE_SIZE];
  size_t bytes_in = 0;

  while (bytes_in < MESSAGE_SIZE) {
    while (Serial.available() > 0) {
      bytes[bytes_in++] = Serial.read();

      if (bytes_in >= MESSAGE_SIZE) {
        break;
      }
    }
  }

  if (recv_drawcall(bytes, &call)) {
    Serial.write("\1"); // ack
  } else {
    Serial.write("\2"); // retransmit
    free(bytes);
    return;
  }

  free(bytes);

  if (call.x == 1 && call.y == 1 && call.color == 0x0101) {
    tft.fillScreen(TFT_BLACK);
    return;
  }

  tft.fillCircle(call.x, call.y, 10, call.color);
}