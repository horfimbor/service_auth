class Login extends HTMLElement {
    constructor() {
        super();

        this.attachShadow({ mode: 'open' });
        this.shadowRoot.innerHTML = `
        <style>
            input{
              background-color: yellow;
            }

            input:invalid {
              border: 2px dashed red;
              background-color: red;
            }

            input:valid {
              border: 1px solid black;
              background-color: blue;
            }
        </style
            <form class="login" method="post">
                <input minlength="5" placeholder="passphrase" type="password" name="passphrase" class="passphrase" required/>
                <hf-button content="login" class="submit"></hf-button>
            </form>
        `
    }

    connectedCallback() {
        this.shadowRoot.querySelector(".submit").addEventListener('click', this._login.bind(this));
    }

    disconnectedCallback() {
        this.shadowRoot.querySelector(".submit").removeEventListener('click', this._login);
    }

    _login(e) {
        e.preventDefault();

        let invalid =  this.shadowRoot.querySelector(":invalid");

        if( invalid){
            return;
        }

        let passphrase = this.shadowRoot.querySelector(".passphrase:valid").value;

        console.debug(passphrase)

        fetch("http://localhost:8000/login", {
                method : "POST",
                headers: {
                   cache : "no-cache"
                },
                body : JSON.stringify({
                    passphrase: passphrase
                }),
            })
            .then(res => res.text()) // parse response as JSON with res.json
            .then(response => {

                if(response === "data_required"){
                       e.preventDefault();
                       var event = new CustomEvent('_auth_signup', { 'detail': passphrase });
                       document.dispatchEvent(event);
                }else{
                    var event = new CustomEvent('_auth_login', { 'detail': response });
                    document.dispatchEvent(event);
                }
            })
            .catch(err => {
                console.log({service:"auth", status:"KO", error:err})
                alert("sorry, cannot auth")
            });
    }
}
customElements.define('hf-auth-login', Login);