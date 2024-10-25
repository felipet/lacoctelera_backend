const tokenBtn = document.getElementById('btn');
const secretToken = document.getElementById('secret');
tokenBtn.addEventListener('click', () => {
  secretToken.classList.toggle('no-filter');
});
