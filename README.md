# OS Embarqué et Edge Computing

Eliot MENORET + Evan GALLI + Élian DELMAS

Code source : https://github.com/Other-Project/SI5-OS-EEC.git

## Introduction

Ce projet consiste en un système d'alarme intelligent pour une maison. En cas de détection d'une présence dans la maison alors que l'alarme est activée, une signal sonore s'activera dans la maison pour effrayer l’intrus. Il sera possible pour l'habitant d'activer son alarme lorsqu'il sort de chez lui, puis de la désactiver en revenant avec un badge RFID personnel.
En plus de cela, l'habitant aura la possibilité d'ajouter de nouveaux badges RFID qu'il pourra donner à ses proches pour que ces derniers puissent eux-même désactiver l'alarme. De manière similaire, il pourra également révoquer des badges RFID pour qu'ils ne puissent plus être utilisés pour désactiver l'alarme. Ces opérations seront réalisées directement au niveau de la Raspberry Pi (Edge).
Ce système comprend :
 - un capteur ultrason pour détecter une présence dans la maison
 - un bouton permettant à l'habitant d'activer son alarme lorsqu'il s'absente
 - un lecteur RFID (récupéré auprès de vous) pour que l'habitant s'identifie et désactive l'alarme lorsqu'il rentre chez lui
 - une LED pour signaler si l'alarme est activée ou non
 - un buzzer qui va émettre un son en cas de détection d'une présence dans la maison lorsque l'alarme est activée
 - un afficheur LCD pour permettre à l'habitant de gérer les badges RFID acceptés pour désactiver l'alarme
 - un potentiomètre et un encodeur de souris (récupéré auprès de vous) utilisé pour naviguer sur l'afficheur LCD entre les modes d'ajout et de révocation de badges RFID
 - un bouton pour sélectionner le mode d'ajout ou de révocation sur l'afficheur LCD pour ajouter ou révoquer un badge RFID
