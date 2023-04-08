#include "Arduino.h"

constexpr uint8_t READINGS_PER_CYCLE(5);

void setup() {
    Serial.begin(9600);

    while (true) {
        Serial.write(0xAAAAAA);
        for (auto i = 0u; i < READINGS_PER_CYCLE; ++i) {
            Serial.write(analogRead(A0));
        }
        delay(5000);
    }
}

void loop() {

}