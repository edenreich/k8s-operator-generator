const { v4: uuidv4 } = require('uuid')

const catNames = [
  'Whiskers',
  'Luna',
  'Bella',
  'Lucy',
  'Oliver',
  'Leo',
  'Milo',
  'Charlie',
  'Simba',
  'Max',
  'Jasper',
  'Shadow',
  'Coco',
  'Smokey',
  'Mittens',
  'Pumpkin',
  'Nala',
  'Biscuit',
  'Paws',
  'Fluffy',
  'Snowball',
  'Tiger',
  'Pepper',
  'Ginger',
  'Felix',
  'Lucky',
  'Oreo',
  'Boots',
  'Daisy',
  'Cinnamon',
]

const catBreeds = [
  'Abyssinian',
  'American Bobtail',
  'American Curl',
  'American Shorthair',
  'American Wirehair',
  'Balinese',
  'Bengal',
  'Birman',
  'Bombay',
  'British Shorthair',
  'Burmese',
  'Burmilla',
  'Chartreux',
  'Chinese Li Hua',
  'Colorpoint Shorthair',
  'Cornish Rex',
  'Cymric',
  'Devon Rex',
]

const random = (arr) => arr[Math.floor(Math.random() * arr.length)]

module.exports = () => {
  const data = []

  for (let i = 0; i < 100; i++) {
    data.push({
      uuid: uuidv4(),
      name: random(catNames),
      breed: random(catBreeds),
      age: Math.floor(Math.random() * 20) + 1,
    })
  }

  return {
    cats: data,
  }
}
